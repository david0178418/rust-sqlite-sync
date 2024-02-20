mod discovery;

use crate::discovery::{query, register};

fn main() {
	// Require a paramter of either "client" or "responder".
	let args: Vec<String> = std::env::args().collect();
	let bar = String::from("<blank>");
	let role = args.get(1).unwrap_or(&bar);

	if role != "client" && role != "responder" {
		println!("Must provide a valid role 'client' or 'responder''");
		std::process::exit(1);
	}

	if role == "client" {
		println!("Running as client");
		query(args[1..].to_vec().clone());
	} else {
		println!("Running as responder");
		register(args[1..].to_vec().clone());

		loop {
			println!("Responder is running");
			std::thread::sleep(std::time::Duration::from_millis(2000));
		}
	}
}
