use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;

use log::LevelFilter;

use cheetah_matches_relay::room::debug::tracer::CommandTracer;
use cheetah_matches_relay::room::template::config::{
	GameObjectTemplatePermission, Permission, PermissionField, PermissionGroup, RoomTemplate, UserTemplate,
};
use cheetah_matches_relay::room::types::FieldType;
use cheetah_matches_relay::server::RelayServer;
use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_matches_relay_common::network::bind_to_free_socket;
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::ObjectOwner;
use cheetah_matches_relay_common::room::{RoomId, UserId, UserPrivateKey};

///
/// Конфигурируем и создаем сервер для интеграционного тестирования
///
#[derive(Debug, Default)]
pub struct IntegrationTestServerBuilder {
	object_id_generator: u32,
	template: RoomTemplate,
	enable_trace: bool,
}

impl IntegrationTestServerBuilder {
	pub const DEFAULT_ACCESS_GROUP: AccessGroups = AccessGroups(55);
	pub const DEFAULT_TEMPLATE: GameObjectTemplateId = 1;

	pub fn set_permission(
		&mut self,
		template: GameObjectTemplateId,
		field_id: FieldId,
		field_type: FieldType,
		group: AccessGroups,
		permission: Permission,
	) {
		let field = PermissionField {
			field_id,
			field_type,
			groups: vec![PermissionGroup { group, permission }],
		};
		match self.template.permissions.templates.iter_mut().find(|tp| tp.template == template) {
			None => self.template.permissions.templates.push(GameObjectTemplatePermission {
				template,
				groups: Default::default(),
				fields: vec![field],
			}),
			Some(template) => {
				template.fields.push(field);
			}
		}
	}

	pub fn enable_trace(&mut self) {
		self.enable_trace = true;
	}

	pub fn build(self) -> (SocketAddr, RelayServer, RoomId) {
		let socket = bind_to_free_socket().unwrap();
		let addr = socket.1;
		let tracer = if self.enable_trace {
			init_logger();
			CommandTracer::new_with_allow_all()
		} else {
			CommandTracer::new_with_deny_all()
		};

		let mut server = RelayServer::new(socket.0, tracer);
		let room_id = server.register_room(self.template).ok().unwrap();
		(addr, server, room_id)
	}
}

#[allow(unused)]
fn init_logger() {
	env_logger::builder()
		.format(|buf, record| {
			writeln!(
				buf,
				"[{}] [{}] {:?}",
				record.level(),
				std::thread::current().name().unwrap_or(""),
				record.args()
			)
		})
		.filter_level(LevelFilter::Info)
		.format_timestamp(None)
		.is_test(true)
		.try_init();
}
