mod queries;

use queries::{Foo, NewTodo, Todo};

#[tauri::command]
fn get_todos() -> Vec<Todo> {
	Foo::new(Some("./foo.db".to_string()))
		.unwrap()
		.fetch_todos()
		.unwrap()
}

#[tauri::command]
fn add_todo(todo: NewTodo) {
	Foo::new(Some("./foo.db".to_string()))
		.unwrap()
		.insert_todo(&todo)
		.unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.invoke_handler(tauri::generate_handler![add_todo, get_todos])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
