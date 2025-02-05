use crate::room::buffer::Buffer;
use crate::room::field::FieldId;
use std::io::Cursor;

use crate::room::object::GameObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryField {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: Buffer,
}

impl BinaryField {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.value.encode(out)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let value = Buffer::decode(input)?;
		Ok(BinaryField { object_id, field_id, value })
	}
}
