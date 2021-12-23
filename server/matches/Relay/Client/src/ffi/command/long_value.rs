use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_long_value_listener(
	client_id: ClientId,
	listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, i64),
) -> u8 {
	execute_with_client(client_id, |client| Ok(client.listener_long_value = Some(listener)))
}

#[no_mangle]
pub extern "C" fn set_long_value(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) -> u8 {
	send_command(
		client_id,
		C2SCommand::SetLong(SetLongCommand {
			object_id: From::from(object_id),
			field_id,
			value,
		}),
	)
}

#[no_mangle]
pub extern "C" fn inc_long_value(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, increment: i64) -> u8 {
	send_command(
		client_id,
		C2SCommand::IncrementLongValue(IncrementLongC2SCommand {
			object_id: From::from(object_id),
			field_id,
			increment,
		}),
	)
}

#[no_mangle]
pub extern "C" fn compare_and_set_long_value(
	client_id: ClientId,
	object_id: &GameObjectIdFFI,
	field_id: FieldId,
	current: i64,
	new: i64,
	reset: i64,
) -> u8 {
	send_command(
		client_id,
		C2SCommand::CompareAndSetLong(CompareAndSetLongCommand {
			object_id: From::from(object_id),
			field_id,
			current,
			new,
			reset,
		}),
	)
}
