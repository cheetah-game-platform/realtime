use crate::server::manager::grpc::proto::realtime_server_management_service_server::RealtimeServerManagementService;
use crate::server::manager::grpc::proto::CreateMemberRequest;
use crate::server::manager::grpc::proto::CreateMemberResponse;
use crate::server::manager::grpc::proto::CreateSuperMemberRequest;
use crate::server::manager::grpc::proto::DeleteMemberRequest;
use crate::server::manager::grpc::proto::DeleteMemberResponse;
use crate::server::manager::grpc::proto::DeleteRoomRequest;
use crate::server::manager::grpc::proto::DeleteRoomResponse;
use crate::server::manager::grpc::proto::EmptyRequest;
use crate::server::manager::grpc::proto::GetRoomsMembersResponse;
use crate::server::manager::grpc::proto::GetRoomsResponse;
use crate::server::manager::grpc::proto::ProbeRequest;
use crate::server::manager::grpc::proto::ProbeResponse;
use crate::server::manager::grpc::proto::RoomIdResponse;
use crate::server::manager::grpc::proto::RoomMembersResponse;
use crate::server::manager::grpc::proto::RoomTemplate;
use crate::server::manager::{ManagementTaskError, ManagementTaskExecutionError};
use crate::server::room::command::ServerCommandError;
use crate::server::room::config::member::MemberCreateParams;
use crate::ServerManager;
use cheetah_common::room::access::AccessGroups;
use cheetah_game_realtime_protocol::others::member_id::MemberAndRoomId;
use cheetah_game_realtime_protocol::RoomId;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use tonic::{Request, Response, Status};

mod from;
pub mod proto;

pub struct RealtimeServerManagementServiceImpl {
	pub server_manager: Arc<Mutex<ServerManager>>,
}

const SUPER_MEMBER_KEY_ENV: &str = "SUPER_MEMBER_KEY";

impl RealtimeServerManagementServiceImpl {
	#[must_use]
	pub fn new(server_manager: Arc<Mutex<ServerManager>>) -> Self {
		RealtimeServerManagementServiceImpl { server_manager }
	}

	async fn register_member(&self, room_id: RoomId, template: MemberCreateParams) -> Result<Response<CreateMemberResponse>, Status> {
		let private_key = template.private_key.clone();
		self.server_manager.lock().await.create_member(room_id, template).map_err(Status::from).map(|member_id| {
			Response::new(CreateMemberResponse {
				user_id: u64::from(member_id),
				private_key: private_key.into(),
			})
		})
	}

	fn create_super_member_if_need(server: &mut MutexGuard<'_, ServerManager>, room_id: RoomId) -> Result<(), ManagementTaskError> {
		if let Ok(key_from_env) = std::env::var(SUPER_MEMBER_KEY_ENV) {
			let key_from_env_bytes = key_from_env.as_bytes();
			let key = key_from_env_bytes.into();
			server.create_member(room_id, MemberCreateParams::new_super_member_with_key(key))?;
		}

		Ok(())
	}
}

#[tonic::async_trait]
impl RealtimeServerManagementService for RealtimeServerManagementServiceImpl {
	async fn create_room(&self, request: Request<RoomTemplate>) -> Result<Response<RoomIdResponse>, Status> {
		let mut server = self.server_manager.lock().await;
		let template = From::from(request.into_inner());
		let room_id = server.create_room(template).map_err(Status::from)?;

		Self::create_super_member_if_need(&mut server, room_id)
			.map(|_| Response::new(RoomIdResponse { room_id }))
			.map_err(Status::from)
	}

	async fn create_member(&self, request: Request<CreateMemberRequest>) -> Result<Response<CreateMemberResponse>, Status> {
		let request = request.into_inner();
		let user_template = request.user.unwrap();
		if user_template.groups == AccessGroups::super_member_group().0 {
			return Err(Status::permission_denied("Wrong member group"));
		}
		self.register_member(request.room_id, MemberCreateParams::from(user_template)).await
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	async fn delete_member(&self, request: Request<DeleteMemberRequest>) -> Result<Response<DeleteMemberResponse>, Status> {
		self.server_manager
			.lock()
			.await
			.delete_member(MemberAndRoomId {
				member_id: request.get_ref().user_id.try_into().map_err(|e| Status::invalid_argument(format!("member_id is too big: {e}")))?,
				room_id: request.get_ref().room_id,
			})
			.map(|_| Response::new(DeleteMemberResponse {}))
			.map_err(Status::from)
	}

	async fn create_super_member(&self, request: Request<CreateSuperMemberRequest>) -> Result<Response<CreateMemberResponse>, Status> {
		let request = request.into_inner();
		self.register_member(request.room_id, MemberCreateParams::new_super_member()).await
	}

	async fn probe(&self, _request: Request<ProbeRequest>) -> Result<Response<ProbeResponse>, Status> {
		Ok(Response::new(ProbeResponse {}))
	}

	async fn get_rooms(&self, _: Request<EmptyRequest>) -> Result<Response<GetRoomsResponse>, Status> {
		self.server_manager
			.lock()
			.await
			.get_rooms()
			.map(|rooms| Response::new(GetRoomsResponse { rooms }))
			.map_err(Status::from)
	}

	async fn delete_room(&self, request: Request<DeleteRoomRequest>) -> Result<Response<DeleteRoomResponse>, Status> {
		let room_id = request.get_ref().id;
		let mut server = self.server_manager.lock().await;
		server.delete_room(room_id).map(|_| Response::new(DeleteRoomResponse {})).map_err(Status::from)
	}

	async fn get_rooms_members(&self, _request: Request<EmptyRequest>) -> Result<Response<GetRoomsMembersResponse>, Status> {
		self.server_manager
			.lock()
			.await
			.get_rooms_member_count()
			.map(|rooms| {
				Response::new(GetRoomsMembersResponse {
					rooms: rooms
						.into_iter()
						.map(|r| RoomMembersResponse {
							room: r.room_id,
							members: r.members.into_iter().map(From::from).collect(),
						})
						.collect(),
				})
			})
			.map_err(Status::from)
	}
}

impl From<ManagementTaskError> for Status {
	fn from(task_err: ManagementTaskError) -> Self {
		match task_err {
			ManagementTaskError::ChannelRecvError(e) => Status::deadline_exceeded(e.to_string()),
			ManagementTaskError::ChannelSendError(e) => Status::unavailable(e.to_string()),
			ManagementTaskError::UnexpectedResultError => Status::internal("unexpected management task result type"),
			ManagementTaskError::TaskExecutionError(ManagementTaskExecutionError::RoomNotFound(e)) => Status::not_found(e.to_string()),
			ManagementTaskError::TaskExecutionError(ManagementTaskExecutionError::UnknownPluginName(e)) => Status::invalid_argument(e),
			ManagementTaskError::TaskExecutionError(ManagementTaskExecutionError::ServerCommandError(server_err)) => match server_err {
				ServerCommandError::MemberNotFound(e) => Status::not_found(e.to_string()),
				ServerCommandError::RoomNotFound(e) => Status::not_found(e.to_string()),
				e => Status::internal(e.to_string()),
			},
		}
	}
}

#[cfg(test)]
mod test {
	use crate::server::manager::grpc::proto::realtime_server_management_service_server::RealtimeServerManagementService;
	use crate::server::manager::grpc::proto::{DeleteMemberRequest, DeleteRoomRequest, EmptyRequest, Member, MemberStatus, RoomMembersResponse};
	use crate::server::manager::grpc::{RealtimeServerManagementServiceImpl, SUPER_MEMBER_KEY_ENV};
	use crate::server::manager::ServerManager;
	use crate::server::room::config::member::MemberCreateParams;
	use cheetah_common::network::bind_to_free_socket;
	use cheetah_game_realtime_protocol::coniguration::ProtocolConfiguration;
	use std::sync::Arc;
	use std::time::Duration;
	use tokio::sync::Mutex;
	use tonic::{Code, Request};

	#[tokio::test]
	async fn should_get_rooms() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeServerManagementServiceImpl::new(Arc::clone(&server_manager));
		let room_1 = server_manager.lock().await.create_room(Default::default()).unwrap();
		let room_2 = server_manager.lock().await.create_room(Default::default()).unwrap();

		let rooms_response = service.get_rooms(Request::new(EmptyRequest::default())).await.unwrap();
		let rooms = rooms_response.get_ref();
		assert!(rooms.rooms.contains(&room_1));
		assert!(rooms.rooms.contains(&room_2));
		assert_eq!(rooms.rooms.len(), 2);
	}

	#[tokio::test]
	async fn should_get_rooms_members_counts() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeServerManagementServiceImpl::new(Arc::clone(&server_manager));
		let room_1 = server_manager.lock().await.create_room(Default::default()).unwrap();
		let room_2 = server_manager.lock().await.create_room(Default::default()).unwrap();
		let member_id = server_manager
			.lock()
			.await
			.create_member(
				room_1,
				MemberCreateParams {
					super_member: false,
					private_key: Default::default(),
					groups: Default::default(),
					objects: vec![],
				},
			)
			.unwrap();

		let rooms_response = service.get_rooms_members(Request::new(EmptyRequest::default())).await.unwrap();
		let rooms = rooms_response.get_ref();
		assert!(rooms.rooms.contains(&RoomMembersResponse {
			room: room_1,
			members: vec![Member {
				id: member_id,
				status: MemberStatus::Created.into()
			}],
		}));
		assert!(rooms.rooms.contains(&RoomMembersResponse {
			room: room_2,
			members: Default::default(),
		}));
		assert_eq!(rooms.rooms.len(), 2);
	}

	#[tokio::test]
	async fn test_create_super_member() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));

		std::env::set_var(SUPER_MEMBER_KEY_ENV, "some-key");
		let service = RealtimeServerManagementServiceImpl::new(Arc::clone(&server_manager));
		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner();

		let dump_response = server_manager.lock().await.dump(room_id.room_id).unwrap();
		assert_eq!(dump_response.unwrap().members.len(), 1);
	}

	#[tokio::test]
	async fn test_delete_room() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));

		let service = RealtimeServerManagementServiceImpl::new(Arc::clone(&server_manager));
		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner().room_id;

		service.delete_room(Request::new(DeleteRoomRequest { id: room_id })).await.unwrap();
	}

	#[tokio::test]
	async fn test_delete_room_not_exist() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));

		let service = RealtimeServerManagementServiceImpl::new(Arc::clone(&server_manager));

		service.delete_room(Request::new(Default::default())).await.unwrap_err();
	}

	#[tokio::test]
	async fn test_delete_member() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeServerManagementServiceImpl::new(Arc::clone(&server_manager));

		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner().room_id;
		let member_id = service.register_member(room_id, MemberCreateParams::default()).await.unwrap().into_inner().user_id;
		assert!(!server_manager.lock().await.dump(room_id).unwrap().unwrap().members.is_empty(), "room should not be empty");

		assert!(
			service.delete_member(Request::new(DeleteMemberRequest { room_id, user_id: member_id })).await.is_ok(),
			"delete_member should return ok"
		);

		assert!(
			!server_manager.lock().await.dump(room_id).unwrap().unwrap().members.iter().any(|u| *u.0 == member_id as u64),
			"deleted member should not be in the room"
		);
	}

	#[tokio::test]
	async fn test_delete_member_room_not_exist() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeServerManagementServiceImpl::new(Arc::clone(&server_manager));

		let res = service.delete_member(Request::new(DeleteMemberRequest { user_id: 0, room_id: 0 })).await;

		assert!(matches!(res.unwrap_err().code(), Code::NotFound), "delete_member should return not_found");
	}

	fn new_server_manager() -> ServerManager {
		ServerManager::new(
			bind_to_free_socket().unwrap(),
			ProtocolConfiguration {
				disconnect_timeout: Duration::from_secs(30),
			},
		)
		.unwrap()
	}
}
