use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, SendError, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use fnv::FnvHashSet;
use thiserror::Error;

use cheetah_protocol::coniguration::ProtocolConfiguration;
use cheetah_protocol::others::member_id::MemberAndRoomId;
use cheetah_protocol::{RoomId, RoomMemberId};

use crate::debug::proto::admin;
use crate::room::command::ServerCommandError;
use crate::room::forward::ForwardConfig;
use crate::room::template::config::{MemberTemplate, Permissions, RoomTemplate};
use crate::room::RoomInfo;
use crate::server::room_registry::RoomNotFoundError;
use crate::server::Server;

///
/// Управление сервером
/// - запуск сервера в отдельном потоке
/// - связь с сервером через Sender
///
pub struct ServerManager {
	sender: Sender<ManagementTaskChannel>,
	halt_signal: Arc<AtomicBool>,
}

#[derive(Debug)]
pub enum ManagementTask {
	CreateRoom(RoomTemplate),
	CreateMember(RoomId, MemberTemplate),
	DeleteMember(MemberAndRoomId),
	Dump(RoomId),
	GetRooms,
	GetRoomsMemberCount,
	DeleteRoom(RoomId),
	PutForwardedCommandConfig(RoomId, ForwardConfig),
	MarkRoomAsReady(RoomId, String),
	GetRoomInfo(RoomId),
	UpdateRoomPermissions(RoomId, Permissions),
}

#[derive(Debug)]
pub enum ManagementTaskResult {
	CreateRoom(RoomId),
	CreateMember(RoomMemberId),
	DeleteMember,
	Dump(admin::DumpResponse),
	GetRooms(Vec<RoomId>),
	GetRoomsMemberCount(Vec<RoomMembersCount>),
	DeleteRoom,
	PutForwardedCommandConfig,
	MarkRoomAsReady,
	GetRoomInfo(RoomInfo),
	UpdateRoomPermissions,
}

#[derive(Debug)]
pub struct RoomMembersCount {
	pub room_id: RoomId,
	pub members: usize,
	pub connected_members: usize,
}

#[derive(Error, Debug)]
pub enum RoomsServerManagerError {
	#[error("CannotCreateServerThread {0}")]
	CannotCreateServerThread(String),
}

#[derive(Error, Debug)]
pub enum ManagementTaskError {
	#[error("ChannelSendError {0}")]
	ChannelSendError(SendError<ManagementTaskChannel>),
	#[error("ChannelRecvError {0}")]
	ChannelRecvError(RecvTimeoutError),
	#[error("TaskExecutionError {0}")]
	TaskExecutionError(ManagementTaskExecutionError),
	#[error("UnexpectedResultError")]
	UnexpectedResultError,
}

#[derive(Error, Debug)]
pub enum ManagementTaskExecutionError {
	#[error("RoomNotFound {0}")]
	RoomNotFound(#[from] RoomNotFoundError),
	#[error("UnknownPluginName {0}")]
	UnknownPluginName(String),
	#[error("ServerCommandError {0}")]
	ServerCommandError(#[from] ServerCommandError),
}

pub struct ManagementTaskChannel {
	pub task: ManagementTask,
	pub sender: Sender<Result<ManagementTaskResult, ManagementTaskExecutionError>>,
}

impl Drop for ServerManager {
	fn drop(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
	}
}

impl ServerManager {
	pub fn new(socket: UdpSocket, plugin_names: FnvHashSet<String>, protocol_configuration: ProtocolConfiguration) -> Result<Self, RoomsServerManagerError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let halt_signal = Arc::new(AtomicBool::new(false));
		let cloned_halt_signal = Arc::clone(&halt_signal);
		thread::Builder::new()
			.name(format!("server({:?})", socket.local_addr()))
			.spawn(move || match Server::new(socket, receiver, halt_signal, plugin_names, protocol_configuration) {
				Ok(server) => {
					server.run();
					Ok(())
				}
				Err(e) => {
					tracing::error!("Error running network thread {:?}", e);
					Err(e)
				}
			})
			.map_err(|e| RoomsServerManagerError::CannotCreateServerThread(format!("{e:?}")))?;
		Ok(Self {
			sender,
			halt_signal: cloned_halt_signal,
		})
	}

	pub(crate) fn get_rooms(&self) -> Result<Vec<RoomId>, ManagementTaskError> {
		self.execute_task(ManagementTask::GetRooms).map(|res| {
			if let ManagementTaskResult::GetRooms(rooms) = res {
				Ok(rooms)
			} else {
				Err(ManagementTaskError::UnexpectedResultError)
			}
		})?
	}

	pub(crate) fn get_rooms_member_count(&self) -> Result<Vec<RoomMembersCount>, ManagementTaskError> {
		self.execute_task(ManagementTask::GetRoomsMemberCount).map(|res| {
			if let ManagementTaskResult::GetRoomsMemberCount(rooms) = res {
				Ok(rooms)
			} else {
				Err(ManagementTaskError::UnexpectedResultError)
			}
		})?
	}

	pub fn create_room(&mut self, template: RoomTemplate) -> Result<RoomId, ManagementTaskError> {
		self.execute_task(ManagementTask::CreateRoom(template)).map(|res| {
			if let ManagementTaskResult::CreateRoom(room_id) = res {
				Ok(room_id)
			} else {
				Err(ManagementTaskError::UnexpectedResultError)
			}
		})?
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	pub fn delete_member(&mut self, id: MemberAndRoomId) -> Result<(), ManagementTaskError> {
		self.execute_task(ManagementTask::DeleteMember(id)).map(|_| ())
	}

	/// удалить комнату с сервера и закрыть соединение со всеми пользователями
	pub fn delete_room(&mut self, room_id: RoomId) -> Result<(), ManagementTaskError> {
		self.execute_task(ManagementTask::DeleteRoom(room_id)).map(|_| ())
	}

	pub fn create_member(&mut self, room_id: RoomId, template: MemberTemplate) -> Result<RoomMemberId, ManagementTaskError> {
		self.execute_task(ManagementTask::CreateMember(room_id, template)).map(|res| {
			if let ManagementTaskResult::CreateMember(id) = res {
				Ok(id)
			} else {
				Err(ManagementTaskError::UnexpectedResultError)
			}
		})?
	}

	pub(crate) fn put_forwarded_command_config(&mut self, room_id: RoomId, config: ForwardConfig) -> Result<(), ManagementTaskError> {
		self.execute_task(ManagementTask::PutForwardedCommandConfig(room_id, config)).map(|_| ())
	}

	pub(crate) fn mark_room_as_ready(&mut self, room_id: RoomId, plugin_name: String) -> Result<(), ManagementTaskError> {
		self.execute_task(ManagementTask::MarkRoomAsReady(room_id, plugin_name)).map(|_| ())
	}

	pub(crate) fn update_room_permissions(&mut self, room_id: RoomId, permissions: Permissions) -> Result<(), ManagementTaskError> {
		self.execute_task(ManagementTask::UpdateRoomPermissions(room_id, permissions)).map(|_| ())
	}

	pub(crate) fn get_room_info(&mut self, room_id: RoomId) -> Result<RoomInfo, ManagementTaskError> {
		self.execute_task(ManagementTask::GetRoomInfo(room_id)).map(|res| {
			if let ManagementTaskResult::GetRoomInfo(room_info) = res {
				Ok(room_info)
			} else {
				Err(ManagementTaskError::UnexpectedResultError)
			}
		})?
	}

	pub(crate) fn dump(&self, room_id: u64) -> Result<admin::DumpResponse, ManagementTaskError> {
		self.execute_task(ManagementTask::Dump(room_id)).map(|res| {
			if let ManagementTaskResult::Dump(resp) = res {
				Ok(resp)
			} else {
				Err(ManagementTaskError::UnexpectedResultError)
			}
		})?
	}

	fn execute_task(&self, task: ManagementTask) -> Result<ManagementTaskResult, ManagementTaskError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ManagementTaskChannel { task, sender }).map_err(ManagementTaskError::ChannelSendError)?;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(Ok(result)) => Ok(result),
			Ok(Err(e)) => Err(ManagementTaskError::TaskExecutionError(e)),
			Err(e) => Err(ManagementTaskError::ChannelRecvError(e)),
		}
	}

	pub(crate) fn get_halt_signal(&self) -> Arc<AtomicBool> {
		Arc::clone(&self.halt_signal)
	}

	pub fn shutdown(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
	}
}

#[cfg(test)]
mod test {
	use std::time::Duration;

	use fnv::FnvHashSet;

	use cheetah_common::network::bind_to_free_socket;
	use cheetah_protocol::coniguration::ProtocolConfiguration;

	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::server::manager::ServerManager;

	#[test]
	fn should_get_rooms() {
		let mut server = new_server_manager();
		let room_id = server.create_room(RoomTemplate::default()).unwrap();
		let rooms = server.get_rooms().unwrap();
		assert_eq!(rooms, vec![room_id]);
	}

	#[test]
	fn should_create_member() {
		let mut server = new_server_manager();
		let room_id = server.create_room(RoomTemplate::default()).unwrap();
		let member_id = server.create_member(room_id, MemberTemplate::default()).unwrap();

		assert_eq!(member_id, 1);
	}

	fn new_server_manager() -> ServerManager {
		ServerManager::new(
			bind_to_free_socket().unwrap(),
			FnvHashSet::default(),
			ProtocolConfiguration {
				disconnect_timeout: Duration::from_secs(30),
			},
		)
		.unwrap()
	}
}
