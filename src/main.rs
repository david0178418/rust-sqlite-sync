mod discovery;

use crate::discovery::{query, register, MDnsService};
use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	thread::{self, sleep},
	time::Duration,
};

fn main() {
	// Require a paramter of either "client" or "responder".
	let args: Vec<String> = std::env::args().collect();
	let bar = String::from("<blank>");
	let role = args.get(1).unwrap_or(&bar);
	// let service_type = "_my-hello._tcp.local.";
	let service = MDnsService {
		instance_name: "test1".to_string(),
		service_name: "my-hello".to_string(),
		protocol: "tcp".to_string(),
	};

	if role != "client" && role != "responder" {
		println!("Must provide a valid role 'client' or 'responder''");
		std::process::exit(1);
	}

	if role == "client" {
		println!("Running as client");
		let run_flag = Arc::new(AtomicBool::new(true));
		let run_flag_clone = Arc::clone(&run_flag);
		println!("Scanning for 5 seconds");
		let handle = thread::spawn(move || query(&service, run_flag_clone));

		sleep(Duration::from_secs(5));

		run_flag.store(false, Ordering::SeqCst);

		let peers = handle.join().unwrap();

		peers.iter().for_each(|p| {
			println!(
				"Peer: {} - {} - {} - {}",
				p.hostname,
				p.port,
				p.addresses.join(", "),
				p.name
			);
		});
	} else {
		println!("Running as responder");
		register(args[1..].to_vec().clone());

		loop {
			println!("Responder is running");
			std::thread::sleep(std::time::Duration::from_millis(2000));
		}
	}
}
