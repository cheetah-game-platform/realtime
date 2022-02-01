use thiserror::Error;

use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::action::DoActionAndSendCommandsError;
use crate::room::{Room, RoomError};

pub mod create;
pub mod created;
pub mod delete;
pub mod event;
pub mod float;
pub mod long;
pub mod room;
pub mod structure;

///
/// Выполнение серверной команды
///
pub trait ServerCommandExecutor {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ExecuteServerCommandError>;
}

#[derive(Error, Debug)]
pub enum ExecuteServerCommandError {
	#[error("{:?}",.0)]
	Error(String),

	#[error("{error:?}")]
	RoomError {
		#[from]
		error: RoomError,
	},

	#[error("Member {member_id:?} not owner for game object {object_id:?}")]
	MemberNotOwnerGameObject {
		object_id: GameObjectId,
		member_id: RoomMemberId,
	},

	#[error("{:?}",.error)]
	DoActionAndSendCommandsError {
		#[from]
		error: DoActionAndSendCommandsError,
	},
}

pub fn execute(command: &C2SCommand, room: &mut Room, user_id: RoomMemberId) -> Result<(), ExecuteServerCommandError> {
	match command {
		C2SCommand::Create(command) => command.execute(room, user_id),
		C2SCommand::SetLong(command) => command.execute(room, user_id),
		C2SCommand::IncrementLongValue(command) => command.execute(room, user_id),
		C2SCommand::CompareAndSetLong(command) => command.execute(room, user_id),
		C2SCommand::SetDouble(command) => command.execute(room, user_id),
		C2SCommand::IncrementDouble(command) => command.execute(room, user_id),
		C2SCommand::SetStructure(command) => command.execute(room, user_id),
		C2SCommand::Event(command) => command.execute(room, user_id),
		C2SCommand::Delete(command) => command.execute(room, user_id),
		C2SCommand::AttachToRoom => room::attach_to_room(room, user_id),
		C2SCommand::DetachFromRoom => room::detach_from_room(room, user_id),
		C2SCommand::Created(command) => command.execute(room, user_id),
		C2SCommand::TargetEvent(command) => command.execute(room, user_id),
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::RoomMemberId;

	use crate::room::template::config::{RoomTemplate, UserTemplate};
	use crate::room::Room;

	pub fn setup_two_players() -> (Room, GameObjectId, RoomMemberId, RoomMemberId) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(0b11);
		let mut room = Room::from_template(template);
		let user_1 = room.register_user(UserTemplate::stub(access_groups));
		let user_2 = room.register_user(UserTemplate::stub(access_groups));
		let object_id = room.create_object(user_1, access_groups).id.clone();
		(room, object_id, user_1, user_2)
	}

	pub fn setup_one_player() -> (Room, RoomMemberId, AccessGroups) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let user_id = room.register_user(UserTemplate::stub(access_groups));
		(room, user_id, access_groups)
	}
}
