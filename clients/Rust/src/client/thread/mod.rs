use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use cheetah_relay_common::commands::command::S2CCommandWithMeta;
use cheetah_relay_common::commands::hash::RoomId;

use crate::client::{Client, NetworkStatus};
use crate::client::request::{ClientRequestType, ExternalRequestProcessor, RequestResult};

pub struct ClientThread {
	client: Client,
	// tcp_client: TCPClient,
	requests: ExternalRequestProcessor,
}

impl ClientThread {
	pub fn new(
		server_address: String,
		room_hash: RoomId,
		client_hash: RoomId,
		receiver: Receiver<ClientRequestType>,
		commands_from_server: Arc<Mutex<Vec<S2CCommandWithMeta>>>,
		network_status: Arc<Mutex<NetworkStatus>>,
	) -> ClientThread {
		ClientThread {
			client: Client::new(room_hash, client_hash, commands_from_server, network_status),
			// tcp_client: TCPClient::new(server_address),
			requests: ExternalRequestProcessor::new(receiver),
		}
	}
	
	pub fn run(&mut self) {
		loop {
			// let network_status = self.tcp_client.cycle(&mut self.client);
			// *self.client.network_status.lock().unwrap() = network_status;
			// match self.requests.cycle(&mut self.client) {
			// 	Ok(result) => {
			// 		match result {
			// 			RequestResult::Ok => {}
			// 			RequestResult::Close => { break; }
			// 		}
			// 	}
			// 	Err(_) => {
			// 		log::error!("client thread error");
			// 		break;
			// 	}
			// }
			// thread::sleep(Duration::from_millis(1));
		}
	}
}