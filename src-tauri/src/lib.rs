mod discovery;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use discovery::{register, MDnsService};
use std::thread::spawn;
use std::{
	io::Error,
	sync::{
		atomic::{AtomicU16, Ordering::Relaxed},
		Arc,
	},
};
use std::{thread::sleep, time::Duration};
use tauri::async_runtime::block_on;

struct Port(u16);

#[tauri::command(async)]
fn broadcast(name: MDnsService) -> Result<(), String> {
	let service = MDnsService { ..name };
	loop {
		sleep(Duration::from_secs(1));
		register(&service, 3003);
	}
}

#[tauri::command]
fn get_port(port: tauri::State<Port>) -> u16 {
	port.0
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let port_arc = Arc::new(AtomicU16::new(0));
	let port_arc_clone = Arc::clone(&port_arc);

	spawn(|| block_on(fiz(port_arc_clone)));

	while port_arc.load(Relaxed) == 0 {
		sleep(Duration::from_millis(100));
	}

	let port = port_arc.load(Relaxed);

	println!("Listening on http://127.0.0.1:{port}");

	spawn(move || {
		register(
			&MDnsService {
				instance_name: format!("test-{port}"),
				service_name: "test".to_string(),
				protocol: "tcp".to_string(),
			},
			port,
		);
	});

	tauri::Builder::default()
		.manage(Port(port))
		.plugin(tauri_plugin_shell::init())
		.invoke_handler(tauri::generate_handler![broadcast, get_port])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

#[get("/")]
async fn hello() -> impl Responder {
	HttpResponse::Ok().body("Hello world!")
}

async fn fiz(p: Arc<AtomicU16>) -> Result<(), Error> {
	let server = HttpServer::new(|| App::new().service(hello))
		.bind(("127.0.0.1", 0))
		.unwrap();

	let selected_port = server.addrs().first().unwrap().port();

	p.store(selected_port, Relaxed);
	server.run().await
}
