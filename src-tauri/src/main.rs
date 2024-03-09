// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod queries;

fn main() {
	println!("Hello, world1!");

	tauri_app_lib::run();
}

// use futures::{executor::block_on, StreamExt};
// use libp2p::{
// 	gossipsub, mdns, noise,
// 	swarm::{NetworkBehaviour, SwarmEvent},
// 	tcp, yamux, Swarm,
// };
// use std::{error::Error, time::Duration};
// use tokio::{io, select};
// use tracing_subscriber::EnvFilter;

// #[derive(NetworkBehaviour)]
// struct MyBehaviour {
// 	gossipsub: gossipsub::Behaviour,
// 	mdns: mdns::tokio::Behaviour,
// }

// const TOPIC: &str = "test-net-2";

// async fn build_swarm() -> Result<Swarm<MyBehaviour>, Box<dyn Error>> {
// 	let _ = tracing_subscriber::fmt()
// 		.with_env_filter(EnvFilter::from_default_env())
// 		.try_init();

// 	let mut swarm = libp2p::SwarmBuilder::with_new_identity()
// 		.with_tokio()
// 		.with_tcp(
// 			tcp::Config::default(),
// 			noise::Config::new,
// 			yamux::Config::default,
// 		)?
// 		.with_quic()
// 		.with_behaviour(|key| {
// 			// Set a custom gossipsub configuration
// 			let gossipsub_config = gossipsub::ConfigBuilder::default()
// 				.heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
// 				.validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
// 				// .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
// 				.build()
// 				.map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

// 			// build a gossipsub network behaviour
// 			let gossipsub = gossipsub::Behaviour::new(
// 				gossipsub::MessageAuthenticity::Signed(key.clone()),
// 				gossipsub_config,
// 			)?;

// 			let mdns =
// 				mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
// 			Ok(MyBehaviour { gossipsub, mdns })
// 		})?
// 		.with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
// 		.build();

// 	let topic = gossipsub::IdentTopic::new(TOPIC);

// 	swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

// 	// Listen on all interfaces and whatever port the OS assigns
// 	swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
// 	swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

// 	loop {
// 		select! {
// 			event = swarm.select_next_some() => match event {
// 				SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
// 					for (peer_id, _multiaddr) in list {
// 						println!("mDNS discovered a new peer: {peer_id}");
// 						swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);

// 					}
// 				},
// 				SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
// 					for (peer_id, _multiaddr) in list {
// 						println!("mDNS discover peer has expired: {peer_id}");
// 						swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
// 					}
// 				},
// 				SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
// 					propagation_source: peer_id,
// 					message_id: id,
// 					message,
// 				})) => println!(
// 					"Got message: '{}' with id: {id} from peer: {peer_id}",
// 					String::from_utf8_lossy(&message.data),
// 				),
// 				_ => {}
// 			}
// 		}
// 	}
// }
