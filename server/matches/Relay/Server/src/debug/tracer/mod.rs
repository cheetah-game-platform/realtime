use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Sender;

use fnv::FnvBuildHasher;
use indexmap::IndexMap;

use cheetah_matches_relay_common::commands::command::{C2SCommand, S2CCommand};
use cheetah_matches_relay_common::constants::GameObjectTemplateId;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::UserId;

use crate::debug::tracer::filter::Filter;
use crate::room::object::GameObject;

///
/// Сервис визуализации потока сетевых команд для отладки
/// adr/matches/0002-relay-debug-commands-flow-in-unity.md
///
///
pub mod filter;
pub mod grpc;
pub mod parser;
pub mod proto;

type SessionId = u16;

///
/// Список сессий для сбора команд
/// Каждая сессия собирает команды в свой буфер и может применять к этому буферу фильтр
/// Подразумевается, что клиент будет создавать сессию и в рамках сессии менять фильтры для уже
/// собранных команд и команд в процессе сбора, тем самым можно анализировать существующие команды
///
#[derive(Debug, Default)]
pub struct CommandTracerSessions {
	session_id_generator: SessionId,
	sessions: HashMap<SessionId, Session>,
}

///
/// Сохраняет все команды с момента создания, также при установке фильтра сохраняет
/// отфильтрованные команды
///
#[derive(Debug, Default)]
struct Session {
	filter: Option<Filter>,
	commands: VecDeque<CollectedCommand>,
	filtered_commands: VecDeque<CollectedCommand>,
}

///
/// Структура для хранения собранной команды
///
#[derive(Debug, Clone, PartialEq)]
pub struct CollectedCommand {
	template: Option<GameObjectTemplateId>,
	user: UserId,
	network_command: UniDirectionCommand,
}
///
/// Хранение команд разной направленности (с сервера на клиент и с клиента на сервер)
///
#[derive(Debug, Clone, PartialEq)]
enum UniDirectionCommand {
	C2S(C2SCommand),
	S2C(S2CCommand),
}

///
/// Ошибка установки фильтра
///
#[derive(Debug)]
pub enum CommandTracerSessionsError {
	QueryError(String),
	SessionNotFound,
}

///
/// Команды к потоку relay сервера
///
pub enum CommandTracerSessionsTask {
	CreateSession(Sender<SessionId>),
	SetFilter(SessionId, String, Sender<Result<(), CommandTracerSessionsError>>),
}

impl Session {
	pub const BUFFER_LIMIT: usize = 65000;

	///
	/// Сохранение сетевой команды
	/// - учитывается ограничение на размер буфера команд
	///
	pub fn collect(&mut self, template: Option<GameObjectTemplateId>, user: UserId, network_command: UniDirectionCommand) {
		let collected_command = CollectedCommand {
			template,
			user,
			network_command,
		};

		match &self.filter {
			None => self.filtered_commands.push_back(collected_command.clone()),
			Some(filter) => {
				if filter.filter(&collected_command) {
					self.filtered_commands.push_back(collected_command.clone())
				}
			}
		}
		self.commands.push_back(collected_command);
		if self.commands.len() >= Session::BUFFER_LIMIT {
			self.commands.pop_front();
		}
		if self.filtered_commands.len() >= Session::BUFFER_LIMIT {
			self.filtered_commands.pop_front();
		}
	}

	///
	/// Сохранить фильтр в сессии и применить его для уже собранных команд
	pub fn apply_filter(&mut self, filter: Filter) {
		let filter = filter;
		self.filtered_commands = self.commands.iter().filter(|c| filter.filter(c)).cloned().collect();
		self.filter = Option::Some(filter);
	}
}

impl CommandTracerSessions {
	///
	/// Создать новую сессию
	///
	pub fn create_session(&mut self) -> SessionId {
		let id = self.session_id_generator;
		self.session_id_generator += 1;
		self.sessions.insert(id, Default::default());
		id
	}

	///
	/// Установить фильтр для сессии
	///
	pub fn set_filter(&mut self, session_id: SessionId, query: String) -> Result<(), CommandTracerSessionsError> {
		match parser::parser().parse(query.as_ref()) {
			Ok(rules) => {
				let filter = Filter::from(rules);
				match self.sessions.get_mut(&session_id) {
					None => Result::Err(CommandTracerSessionsError::SessionNotFound),
					Some(session) => {
						session.apply_filter(filter);
						Result::Ok(())
					}
				}
			}
			Err(e) => Result::Err(CommandTracerSessionsError::QueryError(format!("{:?}", e).to_string())),
		}
	}

	pub fn collect_c2s(&mut self, objects: &IndexMap<GameObjectId, GameObject, FnvBuildHasher>, user: UserId, command: &C2SCommand) {
		self.sessions.values_mut().for_each(|s| {
			let network_command = UniDirectionCommand::C2S(command.clone());
			let template = match network_command.get_object_id() {
				None => Option::None,
				Some(object_id) => {
					let game_object = objects.get(&object_id).unwrap();
					Option::Some(game_object.template.clone())
				}
			};
			s.collect(template, user, network_command);
		})
	}
	pub fn collect_s2c(&mut self, template: GameObjectTemplateId, user: UserId, command: &S2CCommand) {
		self.sessions.values_mut().for_each(|s| {
			let network_command = UniDirectionCommand::S2C(command.clone());
			s.collect(Option::Some(template), user, network_command);
		})
	}

	///
	/// Получить команды из сессии, полученные команды удаляются их отфильтрованных команд
	///
	pub fn drain_filtered_commands(&mut self, session: SessionId) -> Result<Vec<CollectedCommand>, CommandTracerSessionsError> {
		match self.sessions.get_mut(&session) {
			None => Result::Err(CommandTracerSessionsError::SessionNotFound),
			Some(session) => Result::Ok(session.filtered_commands.drain(0..).collect()),
		}
	}

	pub fn do_task(&mut self, task: CommandTracerSessionsTask) {
		match task {
			CommandTracerSessionsTask::CreateSession(sender) => {
				let session_id = self.create_session();
				sender.send(session_id).unwrap();
			}
			CommandTracerSessionsTask::SetFilter(session_id, query, sender) => {
				let result = self.set_filter(session_id, query);
				println!("result {:?}", result);
				sender.send(result).unwrap();
			}
		}
	}
}

#[cfg(test)]
pub mod tests {
	use cheetah_matches_relay_common::commands::command::event::EventCommand;
	use cheetah_matches_relay_common::commands::command::{C2SCommand, S2CCommand};
	use cheetah_matches_relay_common::room::UserId;

	use crate::debug::tracer::{
		CollectedCommand, CommandTracerSessions, CommandTracerSessionsError, CommandTracerSessionsTask, Session, SessionId, UniDirectionCommand,
	};

	#[test]
	fn should_collect_command_without_filter() {
		let mut tracer = CommandTracerSessions::default();
		let session_id = tracer.create_session();
		tracer.collect_c2s(&Default::default(), 100, &C2SCommand::AttachToRoom);
		tracer.collect_c2s(&Default::default(), 101, &C2SCommand::AttachToRoom);
		tracer.collect_c2s(&Default::default(), 102, &C2SCommand::AttachToRoom);
		tracer.collect_s2c(
			200,
			100,
			&S2CCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id: 0,
				event: Default::default(),
			}),
		);
		let commands = tracer.drain_filtered_commands(session_id).unwrap();
		assert_eq!(
			commands,
			vec![
				CollectedCommand {
					template: None,
					user: 100,
					network_command: UniDirectionCommand::C2S(C2SCommand::AttachToRoom)
				},
				CollectedCommand {
					template: None,
					user: 101,
					network_command: UniDirectionCommand::C2S(C2SCommand::AttachToRoom)
				},
				CollectedCommand {
					template: None,
					user: 102,
					network_command: UniDirectionCommand::C2S(C2SCommand::AttachToRoom)
				},
				CollectedCommand {
					template: Some(200),
					user: 100,
					network_command: UniDirectionCommand::S2C(S2CCommand::Event(EventCommand {
						object_id: Default::default(),
						field_id: 0,
						event: Default::default()
					}))
				}
			]
		);
		let commands = tracer.drain_filtered_commands(session_id).unwrap();
		assert!(commands.is_empty());
	}

	#[test]
	fn should_collect_command_with_filter() {
		let mut tracer = CommandTracerSessions::default();
		let session_id = tracer.create_session();
		tracer.collect_c2s(&Default::default(), 100, &C2SCommand::AttachToRoom);
		tracer.collect_c2s(&Default::default(), 101, &C2SCommand::AttachToRoom);
		tracer.collect_c2s(&Default::default(), 102, &C2SCommand::AttachToRoom);
		tracer.collect_s2c(
			200,
			100,
			&S2CCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id: 0,
				event: Default::default(),
			}),
		);
		tracer.set_filter(session_id, "(user=100)".to_string()).unwrap();

		let commands = tracer.drain_filtered_commands(session_id).unwrap();
		assert_eq!(
			commands,
			vec![
				CollectedCommand {
					template: None,
					user: 100,
					network_command: UniDirectionCommand::C2S(C2SCommand::AttachToRoom)
				},
				CollectedCommand {
					template: Some(200),
					user: 100,
					network_command: UniDirectionCommand::S2C(S2CCommand::Event(EventCommand {
						object_id: Default::default(),
						field_id: 0,
						event: Default::default()
					}))
				}
			]
		);
		let commands = tracer.drain_filtered_commands(session_id).unwrap();
		assert!(commands.is_empty());
	}

	#[test]
	fn should_limit_commands_buffer() {
		let mut tracer = CommandTracerSessions::default();
		let session_id = tracer.create_session();
		tracer.collect_c2s(&Default::default(), 50, &C2SCommand::AttachToRoom);
		for _i in 0..Session::BUFFER_LIMIT {
			tracer.collect_c2s(&Default::default(), 1000 as UserId, &C2SCommand::AttachToRoom);
		}
		tracer.collect_c2s(&Default::default(), 55, &C2SCommand::AttachToRoom);

		let session = &mut tracer.sessions.get_mut(&session_id).unwrap();
		assert!(session.filtered_commands.len() < Session::BUFFER_LIMIT);
		assert!(session.commands.len() < Session::BUFFER_LIMIT);
		let last_command = session.commands.pop_back().unwrap();
		assert_eq!(
			last_command,
			CollectedCommand {
				template: None,
				user: 55,
				network_command: UniDirectionCommand::C2S(C2SCommand::AttachToRoom)
			}
		)
	}

	#[test]
	fn should_do_task_create_session() {
		let mut tracer = CommandTracerSessions::default();
		let (sender, receiver) = std::sync::mpsc::channel();
		tracer.do_task(CommandTracerSessionsTask::CreateSession(sender));
		match receiver.try_recv() {
			Ok(session_id) => {
				assert!(tracer.sessions.contains_key(&session_id))
			}
			Err(_) => {
				assert!(false)
			}
		}
	}

	#[test]
	fn should_do_task_set_filter() {
		let mut tracer = CommandTracerSessions::default();
		let session_id = tracer.create_session();
		let (sender, receiver) = std::sync::mpsc::channel();
		tracer.do_task(CommandTracerSessionsTask::SetFilter(session_id, "(user=55)".to_string(), sender));
		match receiver.try_recv() {
			Ok(result) => match result {
				Ok(_) => assert!(true),
				Err(_) => assert!(false),
			},
			Err(_) => assert!(false),
		}
	}

	#[test]
	fn should_do_task_set_wrong_filter() {
		let mut tracer = CommandTracerSessions::default();
		let session_id = tracer.create_session();
		let (sender, receiver) = std::sync::mpsc::channel();
		tracer.do_task(CommandTracerSessionsTask::SetFilter(session_id, "(8=55)".to_string(), sender));
		match receiver.try_recv() {
			Ok(result) => match result {
				Ok(_) => assert!(false),
				Err(_) => assert!(true),
			},
			Err(_) => assert!(false),
		}
	}
}
