use mdns_sd::{DaemonEvent, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::{
	fmt,
	sync::{
		atomic::{AtomicBool, Ordering::Relaxed},
		Arc,
	},
	thread,
	time::Duration,
};

// from https://github.com/keepsimple1/mdns-sd/blob/4d719a3a47152b634a0314bfd9041690772b6e29/examples/query.rs

#[derive(Debug)]
pub struct PeerInfo {
	pub name: String,
	pub hostname: String,
	pub port: u16,
	pub addresses: Vec<String>,
}

pub struct MDnsService {
	pub instance_name: String,
	pub service_name: String,
	pub protocol: String,
}

impl fmt::Display for MDnsService {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{}._{}._{}.local.",
			self.instance_name, self.service_name, self.protocol
		)
	}
}

// test1._my-hello._udp.local.
pub fn query(service: &MDnsService, run_flag: Arc<AtomicBool>) -> Vec<PeerInfo> {
	// Create a daemon
	let mdns = ServiceDaemon::new().expect("Failed to create daemon");
	let mut peers = Vec::<PeerInfo>::new();
	let receiver = mdns.browse(&service.to_string()).expect("Failed to browse");

	while let Ok(event) = receiver.recv_timeout(Duration::from_secs(2)) {
		if let ServiceEvent::ServiceResolved(info) = event {
			peers.push(PeerInfo {
				name: info.get_fullname().to_string(),
				hostname: info.get_hostname().to_string(),
				port: info.get_port(),
				addresses: info.get_addresses().iter().map(|a| a.to_string()).collect(),
			});
		}

		if !run_flag.load(Relaxed) {
			break;
		}
	}

	peers
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
			return;
		},
	};
	let instance_name = match args.get(2) {
		Some(arg) => arg,
		None => {
			return;
		},
	};

	// With `enable_addr_auto()`, we can give empty addrs and let the lib find them.
	// If the caller knows specific addrs to use, then assign the addrs here.
	let my_addrs = "";
	let service_hostname = "mdns-example.local.";
	let port = 3000;

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
