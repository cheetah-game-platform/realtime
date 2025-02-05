use cheetah_game_realtime_protocol::RoomMemberId;

use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::owner::GameObjectOwner;

use crate::server::room::command::ServerCommandError;
use crate::server::room::Room;

pub(crate) fn delete(game_object_id: &GameObjectId, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let member = room.get_member(&member_id)?;
	if let GameObjectOwner::Member(object_id_member) = game_object_id.get_owner() {
		if object_id_member != member.id {
			return Err(ServerCommandError::MemberNotOwnerGameObject {
				object_id: *game_object_id,
				member_id,
			});
		}
	}
	room.delete_object(*game_object_id, member_id)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::command::delete::delete;
	use crate::server::room::command::ServerCommandError;
	use crate::server::room::config::member::MemberCreateParams;
	use crate::server::room::config::room::RoomCreateParams;
	use crate::server::room::Room;

	#[test]
	fn should_delete() {
		let template = RoomCreateParams::default();
		let access_groups = AccessGroups(0b11);

		let mut room = Room::new(0, template);
		let member_a_id = room.register_member(MemberCreateParams::stub(access_groups));
		let member_b_id = room.register_member(MemberCreateParams::stub(access_groups));
		room.mark_as_attached_in_test(member_a_id).unwrap();
		room.mark_as_attached_in_test(member_b_id).unwrap();

		let object_id = room.test_create_object_with_created_state(GameObjectOwner::Member(member_a_id), access_groups, Default::default()).id;
		room.test_out_commands.clear();

		delete(&object_id, &mut room, member_a_id).unwrap();

		assert!(matches!(room.get_object_mut(object_id), Err(_)));
		assert!(matches!(room.get_member_out_commands_for_test(member_a_id).pop_back(), None));
		assert!(matches!(room.get_member_out_commands_for_test(member_b_id).pop_back(), Some(S2CCommand::Delete(c)) if c==object_id));
	}

	#[test]
	fn should_not_delete_if_not_owner() {
		let template = RoomCreateParams::default();
		let access_groups = AccessGroups(55);
		let mut room = Room::new(0, template);
		let member_a = room.register_member(MemberCreateParams::stub(access_groups));
		let member_b = room.register_member(MemberCreateParams::stub(access_groups));

		let object_id = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_a), access_groups, Default::default()).id;
		room.test_out_commands.clear();
		assert!(matches!(delete(&object_id, &mut room, member_b), Err(ServerCommandError::MemberNotOwnerGameObject { .. })));
		assert!(matches!(room.get_object_mut(object_id), Ok(_)));
		assert!(matches!(room.test_out_commands.pop_back(), None));
	}
}
