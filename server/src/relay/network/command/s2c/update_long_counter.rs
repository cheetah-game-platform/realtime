use bytebuffer::ByteBuffer;

use crate::relay::room::objects::object::FieldID;
use crate::relay::room::room::GlobalObjectId;
use crate::relay::network::command::s2c::{AffectedClients, S2CCommand};

#[derive(Debug, PartialEq)]
pub struct UpdateLongCounterS2CCommand {
	pub affected_clients: AffectedClients,
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub value: i64,
}

impl S2CCommand for UpdateLongCounterS2CCommand {
	fn get_command_id(&self) -> u8 {
		3
	}
	
	fn get_affected_clients(&self) -> &AffectedClients {
		return &self.affected_clients;
	}
	
	fn encode(&self, bytes: &mut ByteBuffer) {
		bytes.write_u64(self.global_object_id);
		bytes.write_u16(self.field_id);
		bytes.write_i64(self.value);
	}
}
