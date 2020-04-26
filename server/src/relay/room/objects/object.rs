use std::collections::HashMap;
use std::ops::Shl;

use log::Level::Debug;
use typenum::Gr;

use crate::relay::room::clients::Client;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::listener::RoomListener;
use crate::relay::room::objects::owner::Owner;
use crate::relay::room::room::{GlobalObjectId, Room};

pub type FieldID = u16;
pub type GroupType = u64;

/// Игровой объект
/// содержит данные от пользователей
#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
	pub id: u64,
	pub owner: Owner,
	/// счетчики
	pub long_counters: HashMap<FieldID, LongCounter>,
	pub float_counters: HashMap<FieldID, FloatCounter>,
	/// структуры (для сервера это массивы данных)
	pub structures: HashMap<FieldID, DataStruct>,
	/// группы доступа
	pub groups: AccessGroups,
}

#[derive(Debug, Clone)]
pub struct GameObjectTemplate {
	/// счетчики
	pub long_counters: HashMap<FieldID, LongCounter>,
	pub float_counters: HashMap<FieldID, FloatCounter>,
	/// структуры (для сервера это массивы данных)
	pub structures: HashMap<FieldID, DataStruct>,
	/// группы доступа
	pub groups: AccessGroups,
}


/// счетчик
#[derive(Debug, Clone, PartialEq)]
pub struct LongCounter {
	pub counter: i64
}

/// счетчик
#[derive(Debug, Clone, PartialEq)]
pub struct FloatCounter {
	pub counter: f64
}

/// данные
#[derive(Debug, Clone, PartialEq)]
pub struct DataStruct {
	pub data: Vec<u8>
}

#[derive(Debug)]
pub enum ObjectFieldType {
	LongCounter,
	FloatCounter,
	Struct,
	StringToIdMap,
	IdSet,
	Event,
}


impl GameObject {
	pub fn new_client_object(client: &Client, local_object_id: u32, template: &GameObjectTemplate) -> GameObject {
		GameObject::new(
			GameObject::to_global_object_id(client, local_object_id),
			Owner::new_owner(client),
			template,
		)
	}
	
	pub fn new_root_object(id: u64, template: &GameObjectTemplate) -> GameObject {
		GameObject::new(
			id,
			Owner::new_root_owner(),
			template,
		)
	}
	
	pub fn new(id: u64, owner: Owner, template: &GameObjectTemplate) -> GameObject {
		GameObject {
			id,
			owner: owner.clone(),
			long_counters: template.long_counters.clone(),
			float_counters: template.float_counters.clone(),
			structures: template.structures.clone(),
			groups: template.groups.clone(),
		}
	}
	
	pub fn stub(id: GlobalObjectId) -> GameObject {
		return GameObject::new_root_object(id, &GameObjectTemplate::stub());
	}
	
	pub fn update_struct(&mut self, field_id: FieldID, data: Vec<u8>) {
		self.structures.insert(field_id, DataStruct { data });
	}
	
	pub fn get_struct(&self, field_id: FieldID) -> Option<&Vec<u8>> {
		self.structures.get(&field_id).map(|f| &f.data)
	}
	
	pub fn set_long_counter(&mut self, field_id: FieldID, value: i64) {
		self.long_counters.insert(field_id, LongCounter { counter: value });
	}
	
	pub fn get_long_counter(&self, field_id: FieldID) -> i64 {
		self.long_counters.get(&field_id).map(|f| f.counter).unwrap_or(0)
	}
	
	pub fn increment_long_counter(&mut self, field_id: FieldID, value: i64) -> i64 {
		let new_value = self.get_long_counter(field_id) + value;
		self.set_long_counter(field_id, new_value);
		return new_value;
	}
	
	pub fn set_float_counter(&mut self, field_id: FieldID, value: f64) {
		self.float_counters.insert(field_id, FloatCounter { counter: value });
	}
	
	pub fn get_float_counter(&self, field_id: FieldID) -> f64 {
		self.float_counters.get(&field_id).map(|f| f.counter).unwrap_or(0.0)
	}
	
	pub fn increment_float_counter(&mut self, field_id: FieldID, value: f64) -> f64 {
		let new_value = self.get_float_counter(field_id) + value;
		self.set_float_counter(field_id, new_value);
		return new_value;
	}
	
	
	pub fn send_event(&self, field_id: FieldID, event: &Vec<u8>) {}
	
	pub fn to_global_object_id(client: &Client, local_object_id: u32) -> u64 {
		(client.configuration.id as u64).shl(32) + local_object_id as u64
	}
}


impl Room {
	pub fn object_increment_long_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: i64) -> i64 {
		let result = object.increment_long_counter(field_id, value);
		self.listener.on_object_long_counter_change(field_id, object, &self.clients);
		return result;
	}
	
	pub fn object_increment_float_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: f64) -> f64 {
		let result = object.increment_float_counter(field_id, value);
		self.listener.on_object_float_counter_change(field_id, object, &self.clients);
		return result;
	}
	
	pub fn object_update_struct(&mut self, object: &mut GameObject, field_id: FieldID, value: &Vec<u8>) {
		object.update_struct(field_id, value.clone());
		self.listener.on_object_struct_updated(field_id, object, &self.clients);
	}
	
	pub fn object_send_event(&mut self, object: &mut GameObject, field_id: FieldID, event_data: &Vec<u8>) {
		object.send_event(field_id, event_data);
		self.listener.on_object_event_fired(field_id, &event_data, object, &self.clients);
	}
}


impl GameObjectTemplate {
	pub fn stub() -> GameObjectTemplate {
		GameObjectTemplate {
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			groups: AccessGroups::new(),
		}
	}
	
	/// todo переделать на AccessGroup
	pub fn stub_with_group(group: GroupType) -> GameObjectTemplate {
		GameObjectTemplate {
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			groups: AccessGroups::from(group),
		}
	}
}