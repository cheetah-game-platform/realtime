use core::fmt;
use std::fmt::{Debug, Formatter};

use cheetah_relay_common::constants::MAX_SIZE_STRUCT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Bytes {
	pub count: u8,
	pub value: [u8; MAX_SIZE_STRUCT],
}

impl Bytes {
	pub fn as_slice(&self) -> &[u8] {
		&self.value[0..self.count as usize]
	}
}

impl Debug for Bytes {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f
			.debug_struct("$name")
			.field("size", &self.count)
			.finish()
	}
}

impl Default for Bytes {
	fn default() -> Self {
		Bytes {
			count: 0,
			value: [0; MAX_SIZE_STRUCT],
		}
	}
}

impl From<Vec<u8>> for Bytes {
	fn from(value: Vec<u8>) -> Self {
		let mut result: Bytes = Default::default();
		result.count = value.len() as u8;
		result.value[0..value.len()].copy_from_slice(&value);
		result
	}
}

impl From<Bytes> for Vec<u8> {
	fn from(value: Bytes) -> Self {
		Vec::from(&value.value[0..value.count as usize])
	}
}

#[cfg(test)]
mod tests {
	use crate::client::ffi::bytes::Bytes;
	
	#[test]
	fn should_convert_bytes() {
		let source: Vec<u8> = vec![1, 2, 3, 4, 5];
		let field_ffi_binary = Bytes::from(source.clone());
		let converted = Vec::from(field_ffi_binary.clone());
		assert_eq!(source, converted);
	}
}