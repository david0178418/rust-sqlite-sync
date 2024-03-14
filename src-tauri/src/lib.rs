mod queries;

use queries::{Foo, NewTodo, Todo};

struct StateContainer {
	sender: std::sync::mpsc::Sender<String>,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let (sender, receiver) = channel::<String>();

	spawn(move || {
		let mut swarm = block_on(async { build_swarm().await.unwrap() });

		loop {
			if let Ok(msg) = receiver.recv_timeout(Duration::from_secs(1)) {
				println!("msg received: {}", msg);
			}

			block_on(async {
				select! {
					event = swarm.select_next_some() => match event {
						SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
							for (peer_id, _multiaddr) in list {
								let behavior = swarm.behaviour_mut();
								behavior.gossipsub.add_explicit_peer(&peer_id);

								let _ = behavior.gossipsub.publish(
									peer_id_topic(&peer_id),
									"Direct message of 'hi'".as_bytes()
								);

								let _ = behavior.gossipsub
									.publish(gossipsub::IdentTopic::new(TOPIC), "Hi to all".as_bytes());
								// let (_x) = swarm.dial(peer_id.clone()).unwrap();
							}
						},
						SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
							for (peer_id, _multiaddr) in list {
								println!("mDNS discover peer has expired: {peer_id}");
								swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
							}
						},
						SwarmEvent::NewListenAddr { address, .. } => println!("NewListenAddr: {address:?}"),
						SwarmEvent::IncomingConnectionError { error, .. } => println!("IncomingConnectionError: {error:?}"),
						SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
							propagation_source,
							message_id,
							message,
						})) => println!(
							"Got message: '{}' with id: {message_id} from peer: {propagation_source}",
							String::from_utf8_lossy(&message.data),
						),
						e => {
							println!("Unhandled event: {:?}", e);
						}
					}
				}
			})
		}
	});

	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.manage(StateContainer { sender })
		.invoke_handler(tauri::generate_handler![add_todo, get_todos, delete_todo])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

#[tauri::command]
fn get_todos() -> Vec<Todo> {
	Foo::new(Some("./foo.db".to_string()))
		.unwrap()
		.fetch_todos()
		.unwrap()
}

#[tauri::command]
fn add_todo(state: tauri::State<StateContainer>, todo: NewTodo) {
	Foo::new(Some("./foo.db".to_string()))
		.unwrap()
		.insert_todo(&todo)
		.unwrap();

	state.sender.send("test".to_string()).unwrap();
	println!("added todo: {:?}", todo);
}

#[tauri::command]
fn delete_todo(id: String) {
	Foo::new(Some("./foo.db".to_string()))
		.unwrap()
		.delete_todo(&id)
		.unwrap();
}

use futures::StreamExt;
use libp2p::{
	gossipsub, mdns, noise,
	swarm::{NetworkBehaviour, SwarmEvent},
	tcp, yamux, PeerId, Swarm,
};
use std::{error::Error, sync::mpsc::channel, thread::spawn, time::Duration};
use tauri::async_runtime::block_on;
use tokio::{io, select};
use tracing_subscriber::EnvFilter;

#[derive(NetworkBehaviour)]
struct MyBehaviour {
	gossipsub: gossipsub::Behaviour,
	mdns: mdns::tokio::Behaviour,
}

const TOPIC: &str = "test-net-2";

#[allow(unused)]
async fn build_swarm() -> Result<Swarm<MyBehaviour>, Box<dyn Error>> {
	let _ = tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.try_init();

	let mut swarm = libp2p::SwarmBuilder::with_new_identity()
		.with_tokio()
		.with_tcp(
			tcp::Config::default(),
			noise::Config::new,
			yamux::Config::default,
		)?
		.with_quic()
		.with_behaviour(|key| {
			// Set a custom gossipsub configuration
			let gossipsub_config = gossipsub::ConfigBuilder::default()
				.heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
				.validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
				// .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
				.build()
				.map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

			// build a gossipsub network behaviour
			let mut gossipsub = gossipsub::Behaviour::new(
				gossipsub::MessageAuthenticity::Signed(key.clone()),
				gossipsub_config,
			)?;

			let my_peer_id = key.public().to_peer_id();

			let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), my_peer_id)?;

			gossipsub.subscribe(&gossipsub::IdentTopic::new(TOPIC))?;
			gossipsub.subscribe(&peer_id_topic(&my_peer_id))?;

			Ok(MyBehaviour { gossipsub, mdns })
		})?
		.with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
		.build();

	// let behaviour = swarm.behaviour_mut();

	// let topic_all = gossipsub::IdentTopic::new(TOPIC);
	// let topic_direct = peer_id_topic(&x);

	// behaviour.gossipsub.subscribe(&topic_all)?;
	// behaviour.gossipsub.subscribe(&topic_direct)?;

	// Listen on all interfaces and whatever port the OS assigns
	swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
	swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

	Ok(swarm)
}

fn peer_id_topic(peer_id: &PeerId) -> gossipsub::IdentTopic {
	println!("peer_id_topic: {:?}", format!("{}-{}", TOPIC, peer_id));
	gossipsub::IdentTopic::new(format!("{}-{}", TOPIC, peer_id))
}
