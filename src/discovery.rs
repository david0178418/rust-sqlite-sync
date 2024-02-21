use mdns_sd::{DaemonEvent, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::{
	fmt,
	sync::{
		atomic::{AtomicBool, Ordering::Relaxed},
		Arc,
	},
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
		write!(f, "_{}._{}.local.", self.service_name, self.protocol)
	}
}

pub fn query(service: &MDnsService, run_flag: Arc<AtomicBool>) -> Vec<PeerInfo> {
	// Create a daemon
	let mdns = ServiceDaemon::new().expect("Failed to create daemon");
	let mut peers = Vec::<PeerInfo>::new();
	println!("Browsing for service: {}", &service.to_string());
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

pub fn register(service: &MDnsService, port: u16) {
	// Create a new mDNS daemon.
	let mdns = ServiceDaemon::new().expect("Could not create service daemon");

	// With `enable_addr_auto()`, we can give empty addrs and let the lib find them.
	// If the caller knows specific addrs to use, then assign the addrs here.
	let my_addrs = "";

	// The key string in TXT properties is case insensitive. Only the first
	// (key, val) pair will take effect.
	let properties = [("PATH", "one"), ("Path", "two"), ("PaTh", "three")];

	// Register a service.
	let service_info = ServiceInfo::new(
		&service.to_string(),
		&service.instance_name,
		&service.instance_name,
		my_addrs,
		port,
		&properties[..],
	)
	.expect("valid service info")
	.enable_addr_auto();

	// Optionally, we can monitor the daemon events.
	let monitor = mdns.monitor().expect("Failed to monitor the daemon");
	mdns.register(service_info)
		.expect("Failed to register mDNS service");

	println!("Registered service {}", &service.to_string());

	// Monitor the daemon events.
	while let Ok(event) = monitor.recv() {
		println!("Daemon event: {:?}", &event);
		if let DaemonEvent::Error(e) = event {
			println!("Failed: {}", e);
			break;
		}
	}
}
