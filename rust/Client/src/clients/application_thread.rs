use cheetah_game_realtime_protocol::RoomMemberId;
use std::slice;
use std::sync::mpsc::{Receiver, SendError, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::clients::network_thread::C2SCommandWithChannel;
use crate::clients::{ClientRequest, SharedClientStatistics};
use crate::ffi::channel::Channel;
use crate::ffi::command::S2CCommandFFI;
use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::guarantees::{ChannelGroup, ReliabilityGuarantees};
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::create::CreateGameObject;
use cheetah_common::commands::{BothDirectionCommand, CommandTypeId, CommandWithReliabilityGuarantees};
use cheetah_common::network::ConnectionStatus;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::owner::GameObjectOwner;

///
/// Взаимодействие с сетевым потоком клиента, через Sender
///
pub struct ApplicationThreadClient {
	member_id: RoomMemberId,
	s2c_receiver: Receiver<CommandWithReliabilityGuarantees>,
	pub(crate) handler: Option<JoinHandle<()>>,
	state: Arc<Mutex<ConnectionStatus>>,
	server_time: Arc<Mutex<Option<u64>>>,
	pub(crate) request_to_client: Sender<ClientRequest>,
	channel: ReliabilityGuarantees,
	game_object_id_generator: u32,
	pub shared_statistics: SharedClientStatistics,
}

impl Drop for ApplicationThreadClient {
	fn drop(&mut self) {
		self.handler.take().unwrap().join().unwrap();
	}
}

impl ApplicationThreadClient {
	pub fn new(
		member_id: RoomMemberId,
		handler: JoinHandle<()>,
		state: Arc<Mutex<ConnectionStatus>>,
		in_commands: Receiver<CommandWithReliabilityGuarantees>,
		sender: Sender<ClientRequest>,
		shared_statistics: SharedClientStatistics,
		server_time: Arc<Mutex<Option<u64>>>,
	) -> Self {
		Self {
			member_id,
			s2c_receiver: in_commands,
			handler: Some(handler),
			state,
			server_time,
			request_to_client: sender,
			channel: ReliabilityGuarantees::ReliableSequence(ChannelGroup(0)),
			game_object_id_generator: GameObjectId::CLIENT_OBJECT_ID_OFFSET,
			shared_statistics,
		}
	}

	pub fn set_protocol_time_offset(&mut self, time_offset: Duration) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::SetProtocolTimeOffsetForTest(time_offset))
	}

	pub fn send(&mut self, command: C2SCommand) -> Result<(), SendError<ClientRequest>> {
		let command_with_channel = C2SCommandWithChannel { channel_type: self.channel, command };
		tracing::debug!("c2s {:?}", command_with_channel);
		self.request_to_client.send(ClientRequest::SendCommandToServer(command_with_channel))
	}

	pub fn get_connection_status(&self) -> Result<ConnectionStatus, PoisonError<MutexGuard<'_, ConnectionStatus>>> {
		Ok(self.state.lock()?.clone())
	}

	#[allow(clippy::unwrap_in_result)]
	pub fn get_server_time(&self) -> Option<u64> {
		*self.server_time.lock().unwrap()
	}

	pub fn set_current_channel(&mut self, channel: Channel, group: ChannelGroup) {
		self.channel = match channel {
			Channel::ReliableUnordered => ReliabilityGuarantees::ReliableUnordered,
			Channel::UnreliableUnordered => ReliabilityGuarantees::UnreliableUnordered,
			Channel::ReliableOrdered => ReliabilityGuarantees::ReliableOrdered(group),
			Channel::UnreliableOrdered => ReliabilityGuarantees::UnreliableOrdered(group),
			Channel::ReliableSequence => ReliabilityGuarantees::ReliableSequence(group),
		}
	}

	pub unsafe fn receive(&mut self, commands: *mut S2CCommandFFI, count: &mut u16) {
		*count = 0;
		let commands: &mut [S2CCommandFFI] = slice::from_raw_parts_mut(commands, 1024);

		while let Ok(command) = self.s2c_receiver.try_recv() {
			tracing::debug!("s2c {:?}", command);
			if let BothDirectionCommand::S2C(command) = command.command {
				let command_ffi = &mut commands[*count as usize];
				match command {
					S2CCommand::Create(command) => {
						command_ffi.command_type = CommandTypeId::CreateGameObject;
						command_ffi.command.create = command;
					}
					S2CCommand::Created(command) => {
						command_ffi.command_type = CommandTypeId::CreatedGameObject;
						command_ffi.command.created = command;
					}

					S2CCommand::SetLong(command) => {
						command_ffi.command_type = CommandTypeId::SetLong;
						command_ffi.command.set_long = command;
					}
					S2CCommand::SetDouble(command) => {
						command_ffi.command_type = CommandTypeId::SetDouble;
						command_ffi.command.set_double = command;
					}
					S2CCommand::SetStructure(command) => {
						command_ffi.command_type = CommandTypeId::SetStructure;
						command_ffi.command.buffer_field = command.into();
					}

					S2CCommand::Event(command) => {
						command_ffi.command_type = CommandTypeId::SendEvent;
						command_ffi.command.buffer_field = command.into();
					}
					S2CCommand::Delete(command) => {
						command_ffi.command_type = CommandTypeId::DeleteObject;
						command_ffi.command.game_object_id = command;
					}
					S2CCommand::DeleteField(command) => {
						command_ffi.command_type = CommandTypeId::DeleteField;
						command_ffi.command.delete_field = command;
					}
					S2CCommand::MemberConnected(command) => {
						command_ffi.command_type = CommandTypeId::MemberConnected;
						command_ffi.command.member_connect = command;
					}
					S2CCommand::MemberDisconnected(command) => {
						command_ffi.command_type = CommandTypeId::MemberDisconnected;
						command_ffi.command.member_disconnect = command;
					}
					S2CCommand::AddItem(command) => {
						command_ffi.command_type = CommandTypeId::AddItem;
						command_ffi.command.buffer_field = command.into();
					}
				}
				*count += 1;
				if *count == 1024 {
					break;
				}
			}
		}
	}

	pub fn create_game_object(&mut self, template: u16, access_group: u64) -> Result<GameObjectId, SendError<ClientRequest>> {
		self.game_object_id_generator += 1;
		let game_object_id = GameObjectId::new(self.game_object_id_generator, GameObjectOwner::Member(self.member_id));
		self.send(C2SCommand::CreateGameObject(CreateGameObject {
			object_id: game_object_id,
			template,
			access_groups: AccessGroups(access_group),
		}))?;

		Ok(game_object_id)
	}

	pub fn set_rtt_emulation(&mut self, rtt: Duration, rtt_dispersion: f64) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ConfigureRttEmulation(rtt, rtt_dispersion))
	}

	pub fn set_drop_emulation(&mut self, drop_probability: f64, drop_time: Duration) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ConfigureDropEmulation(drop_probability, drop_time))
	}

	pub fn reset_emulation(&mut self) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ResetEmulation)
	}

	pub fn attach_to_room(&mut self) -> Result<(), SendError<ClientRequest>> {
		// удаляем все пришедшие команды (ситуация возникает при attach/detach)
		while self.s2c_receiver.try_recv().is_ok() {}
		self.send(C2SCommand::AttachToRoom)
	}
}
