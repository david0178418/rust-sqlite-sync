// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use std::thread::spawn;
use tauri::async_runtime::block_on;

#[actix_web::main]
async fn main() {
	println!("Hello, world1!");

	spawn(|| block_on(fuz()));

	tauri_app_lib::run();
	async fn fuz() {
		println!("Hello, world2!");
		let _ = HttpServer::new(|| App::new().service(hello))
			.bind(("127.0.0.1", 3412))
			.unwrap()
			.run()
			.await;
	}
}

#[get("/")]
async fn hello() -> impl Responder {
	HttpResponse::Ok().body("Hello world!")
}
