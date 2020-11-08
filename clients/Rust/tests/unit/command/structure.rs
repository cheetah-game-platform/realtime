use cheetah_relay_client::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_client::client::ffi::bytes::Bytes;
use cheetah_relay_common::commands::command::structure::StructureCommand;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;
use cheetah_relay_common::commands::command::C2SCommandUnion;

#[test]
fn should_to_ffi() {
	let object_id = GameObjectId::new(100, ClientOwner::Root);
	let command = StructureCommand {
		object_id: object_id.clone(),
		field_id: 10,
		structure: vec![1, 2, 3, 4, 5],
	};
	
	let mut ffi = Command::default();
	command.to_ffi(&mut ffi);
	
	assert_eq!(S2CCommandFFIType::Structure, ffi.command_type_s2c);
	assert_eq!(object_id, ffi.object_id.to_common_game_object_id());
	assert_eq!(vec![1 as u8, 2, 3, 4, 5].as_slice(), ffi.structure.as_slice())
}

#[test]
fn should_from_ffi() {
	let object_id = GameObjectId::new(100, ClientOwner::Root);
	let mut ffi = Command::default();
	ffi.command_type_c2s = C2SCommandFFIType::Structure;
	ffi.object_id.set_from(&object_id);
	ffi.field_id = 10;
	ffi.structure = Bytes::from(vec![1, 2, 3]);
	let command = StructureCommand::from_ffi(&ffi);
	assert!(matches!(&command,C2SCommandUnion::Structure(ref structure) if structure.object_id == object_id));
	assert!(matches!(&command,C2SCommandUnion::Structure(ref structure) if structure.field_id == 10));
	assert!(matches!(&command,C2SCommandUnion::Structure(ref structure) if structure.structure.as_slice() == vec![1,2,3].as_slice()));
}