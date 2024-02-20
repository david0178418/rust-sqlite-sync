use mdns_sd::{DaemonEvent, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::{thread, time::Duration};

// from https://github.com/keepsimple1/mdns-sd/blob/4d719a3a47152b634a0314bfd9041690772b6e29/examples/query.rs

pub fn query(args: Vec<String>) {
	// Create a daemon
	let mdns = ServiceDaemon::new().expect("Failed to create daemon");

	let mut service_type: String = match args.get(1) {
		Some(arg) => arg.clone(),
		None => {
			print_query_usage();
			return;
		},
	};

	// Browse for a service type.
	service_type.push_str(".local.");
	let receiver = mdns.browse(&service_type).expect("Failed to browse");

	let now = std::time::Instant::now();
	while let Ok(event) = receiver.recv() {
		match event {
			ServiceEvent::ServiceResolved(info) => {
				println!(
					"Resolved a new service: {}\n\thost: {}\n\tport: {}\n\tIP: {:?}\n\tTXT properties: {:?}",
					info.get_fullname(),
					info.get_hostname(),
					info.get_port(),
					info.get_addresses(),
					info.get_properties(),
				);
			},
			other_event => {
				println!("At {:?} : {:?}", now.elapsed(), &other_event);
			},
		}
	}
}

pub fn register(args: Vec<String>) {
	// Simple command line options.
	let mut should_unreg = false;
	for arg in args.iter() {
		if arg.as_str() == "--unregister" {
			should_unreg = true;
		}
	}

	// Create a new mDNS daemon.
	let mdns = ServiceDaemon::new().expect("Could not create service daemon");
	let service_type = match args.get(1) {
		Some(arg) => format!("{}.local.", arg),
		None => {
			print_register_usage();
			return;
		},
	};
	let instance_name = match args.get(2) {
		Some(arg) => arg,
		None => {
			print_register_usage();
			return;
		},
	};

	// With `enable_addr_auto()`, we can give empty addrs and let the lib find them.
	// If the caller knows specific addrs to use, then assign the addrs here.
	let my_addrs = "";
	let service_hostname = "mdns-example.local.";
	let port = 3456;

	// The key string in TXT properties is case insensitive. Only the first
	// (key, val) pair will take effect.
	let properties = [("PATH", "one"), ("Path", "two"), ("PaTh", "three")];

	// Register a service.
	let service_info = ServiceInfo::new(
		&service_type,
		instance_name,
		service_hostname,
		my_addrs,
		port,
		&properties[..],
	)
	.expect("valid service info")
	.enable_addr_auto();

	// Optionally, we can monitor the daemon events.
	let monitor = mdns.monitor().expect("Failed to monitor the daemon");
	let service_fullname = service_info.get_fullname().to_string();
	mdns.register(service_info)
		.expect("Failed to register mDNS service");

	println!("Registered service {}.{}", &instance_name, &service_type);

	if should_unreg {
		let wait_in_secs = 2;
		println!("Sleeping {} seconds before unregister", wait_in_secs);
		thread::sleep(Duration::from_secs(wait_in_secs));

		let receiver = mdns.unregister(&service_fullname).unwrap();
		while let Ok(event) = receiver.recv() {
			println!("unregister result: {:?}", &event);
		}
	} else {
		// Monitor the daemon events.
		while let Ok(event) = monitor.recv() {
			println!("Daemon event: {:?}", &event);
			if let DaemonEvent::Error(e) = event {
				println!("Failed: {}", e);
				break;
			}
		}
	}
}

fn print_register_usage() {
	println!("Usage:");
	println!("cargo run register <service_type> <instance_name> [--unregister]");
	println!("Options:");
	println!("--unregister: automatically unregister after 2 seconds\n");
	println!("For example:");
	println!("cargo run register _my-hello._udp test1");
}

fn print_query_usage() {
	println!("Usage: cargo run query <service_type_without_domain_postfix>");
	println!("Example: ");
	println!("cargo run query _my-service._udp\n");
	println!("You can also do a meta-query per RFC 6763 to find which services are available:");
	println!("cargo run query _services._dns-sd._udp");
}
