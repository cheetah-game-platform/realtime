use std::cmp::max;
use std::net::UdpSocket;
use std::ops::{Add, Sub};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use admin::DumpResponse;

use crate::debug::proto::admin;
use crate::server::manager::ManagementTask::TimeOffset;
use crate::server::manager::{CommandTracerSessionTaskError, ManagementTask};
use crate::server::rooms::Rooms;
use crate::server::udp::UDPServer;

///
/// Relay сервер, запускается в отдельном потоке, обрабатывает сетевые команды, поддерживает
/// одновременно несколько комнат
///
pub struct Relay {
	udp_server: UDPServer,
	pub rooms: Rooms,
	receiver: Receiver<ManagementTask>,
	max_cycle_time: u128,
	avg_cycle_time: u128,
	halt_signal: Arc<AtomicBool>,
	time_offset: Option<Duration>,
}

impl Drop for Relay {
	fn drop(&mut self) {
		log::error!("Relay: Drop invoked");
	}
}

impl Relay {
	pub fn new(socket: UdpSocket, receiver: Receiver<ManagementTask>, halt_signal: Arc<AtomicBool>) -> Self {
		Self {
			udp_server: UDPServer::new(socket).unwrap(),
			rooms: Rooms::default(),
			receiver,
			max_cycle_time: 0,
			avg_cycle_time: 0,
			halt_signal,
			time_offset: None,
		}
	}

	pub fn run(&mut self) {
		while !self.halt_signal.load(Ordering::Relaxed) {
			let mut now = Instant::now();
			if let Some(time_offset) = self.time_offset {
				now = now.add(time_offset);
			}
			self.udp_server.cycle(&mut self.rooms, &now);
			self.rooms.cycle(&now);
			self.execute_management_tasks();
			self.statistics(now);
		}
	}

	fn execute_management_tasks(&mut self) {
		while let Ok(request) = self.receiver.try_recv() {
			match request {
				ManagementTask::RegisterRoom(template, sender) => {
					let listener = self.udp_server.get_room_user_listener();
					let result = self.rooms.create_room(template.clone(), vec![listener]);
					match sender.send(result) {
						Ok(_) => {}
						Err(e) => {
							log::error!("[Request::RegisterRoom] error send response {:?}", e);
						}
					}
				}
				ManagementTask::RegisterUser(room_id, user_template, sender) => {
					let register_user_result = self.rooms.register_user(room_id, user_template);
					if let Err(e) = sender.send(register_user_result) {
						log::error!("[Request::RegisterUser] error send response {:?}", e);
					}
				}
				TimeOffset(time_offset) => {
					self.time_offset = Option::Some(time_offset);
				}
				ManagementTask::Dump(room_id, sender) => match self.rooms.room_by_id.get(&room_id) {
					None => {
						if let Err(e) = sender.send(Result::Err(format!("Room not found {:?}", room_id).to_string())) {
							log::error!("[Request::Dump] error send response {:?}", e);
						}
					}
					Some(room) => {
						let response: DumpResponse = DumpResponse::from(room);
						let result = Result::Ok(response);
						if let Err(e) = sender.send(result) {
							log::error!("[Request::Dump] error send response {:?}", e);
						}
					}
				},
				ManagementTask::GetRooms(sender) => match sender.send(self.rooms.room_by_id.keys().cloned().collect()) {
					Ok(_) => {}
					Err(e) => {
						log::error!("[Request::RegisterUser] error send response {:?}", e);
					}
				},
				ManagementTask::CommandTracerSessionTask(room_id, task, sender) => {
					match self.rooms.room_by_id.get_mut(&room_id) {
						None => sender
							.send(Result::Err(CommandTracerSessionTaskError::RoomNotFound(room_id)))
							.unwrap(),
						Some(room) => {
							room.command_trace_session.clone().borrow_mut().execute_task(task);
							if let Err(e) = sender.send(Result::Ok(())) {
								log::error!("[Request::RegisterUser] error send response {:?}", e);
							}
						}
					}
				}
			}
		}
	}

	fn statistics(&mut self, start_instant: Instant) {
		let end_instant = Instant::now();
		let duration = end_instant.sub(start_instant).as_millis();
		if duration < 2 {
			thread::sleep(Duration::from_millis(1));
		}
		if self.avg_cycle_time == 0 {
			self.avg_cycle_time = duration;
		} else {
			self.avg_cycle_time = (self.avg_cycle_time + duration) / 2;
		}
		self.max_cycle_time = max(self.max_cycle_time, duration);
	}
}
