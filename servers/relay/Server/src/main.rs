extern crate stderrlog;

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use clap::{App, Arg};
use log::LevelFilter;
use stderrlog::Timestamp;

use cheetah_relay::room::template::RoomTemplate;
use cheetah_relay::server::rest::DumpRestServer;
use cheetah_relay::server::Server;

fn main() {
	let matches = App::new("Cheetah Relay Server")
		.version("0.0.1")
		.about("Realtime multiplayer game server.")
		.arg(
			Arg::new("room-template")
				.long("room")
				.multiple(true)
				.short('r')
				.required(true)
				.about("Config for rooms template (see Server/examples/)")
				.takes_value(true),
		)
		.arg(
			Arg::new("log-level")
				.long("log-level")
				.multiple(false)
				.short('l')
				.required(false)
				.default_value("ERROR")
				.possible_values(&vec!["TRACE", "DEBUG", "INFO", "WARN", "ERROR"])
				.about("level for log")
				.takes_value(true),
		)
		.get_matches();

	let level = match matches.values_of("log-level") {
		None => LevelFilter::Error,
		Some(log_level) => {
			let level_opt = log_level.into_iter().next().unwrap();
			match level_opt {
				"TRACE" => LevelFilter::Trace,
				"DEBUG" => LevelFilter::Debug,
				"INFO" => LevelFilter::Info,
				"WARN" => LevelFilter::Warn,
				"ERROR" => LevelFilter::Error,
				_ => LevelFilter::Error,
			}
		}
	};

	init_logger(level);

	match matches.values_of("room-template") {
		None => {}
		Some(room_templates_path) => {
			start_server(room_templates_path);
		}
	}
}

fn start_server(room_templates_path: clap::Values) {
	let socket = UdpSocket::bind(SocketAddr::from_str("0.0.0.0:5000").unwrap()).unwrap();
	let mut server = Server::new(socket);

	room_templates_path.for_each(|path| {
		let room_template = RoomTemplate::load_from_file(path).unwrap();
		server.register_room(room_template).ok().unwrap();
	});

	let halt_signal = server.get_halt_signal().clone();
	let server = Arc::new(Mutex::new(server));
	DumpRestServer::run(server.clone()).join().unwrap().unwrap();
	halt_signal.store(true, Ordering::Relaxed);
}

fn init_logger(verbosity: LevelFilter) {
	stderrlog::new()
		.verbosity(verbosity as usize)
		.show_level(true)
		.timestamp(Timestamp::Second)
		.init()
		.unwrap();
}
