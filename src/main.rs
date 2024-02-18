use core::panic;
use rusqlite::{params, Connection, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value as Val;
use serde_rusqlite::from_rows;
use std::env;

fn main() -> Result<()> {
	let db_name = &get_db_name(env::args().collect());

	let conn = match get_db_connection(db_name) {
		Ok(conn) => conn,
		Err(e) => panic!("Error: {}", e),
	};
	let max_id = match fetch_table_max_id("todos", &conn) {
		Ok(id) => id,
		Err(e) => panic!("Error: {}", e),
	};

	println!("Max ID: {}", max_id);

	insert_todo_values(max_id + 1, db_name, &conn)
}

fn get_db_name(args: Vec<String>) -> String {
	let db_name = match args.get(1) {
		Some(name) => name,
		None => "test.db",
	};

	if db_name.ends_with(".db") {
		db_name.to_string()
	} else {
		format!("{}.db", db_name)
	}
}

fn get_db_connection(db_name: &String) -> Result<Connection> {
	let conn = Connection::open(db_name)?;

	println!("Loading extension... ");

	unsafe {
		conn.load_extension_enable()?;
		conn.load_extension("./crsqlite.so", Some("sqlite3_crsqlite_init"))?;
		conn.load_extension_disable()?;
	}

	// load and execute init.sql file
	let init_sql = include_str!("init.sql");

	conn.execute_batch(init_sql)?;

	Ok(conn)
}

fn fetch_table_max_id(table_name: &str, conn: &Connection) -> Result<i64> {
	let mut stmt = match conn.prepare(&format!("SELECT MAX(ID) as MAX FROM {};", table_name)) {
		Ok(stmt) => stmt,
		Err(e) => panic!("Error preparing statement: {}", e),
	};
	let mut rows = stmt.query([])?;

	let row = match rows.next() {
		Ok(Some(row)) => row,
		Ok(None) => panic!("Error: no rows found"),
		Err(e) => panic!("Error: no rows found: {}", e),
	};

	let max_id = match row.get::<_, i64>(0) {
		Ok(id) => id,
		Err(e) => {
			println!("Error: max_id{}", e);
			0
		},
	};

	Ok(max_id)
}

struct Todo {
	id: i64,
	label: String,
}

fn add_todo(todo: &Todo, conn: &Connection) -> Result<()> {
	match conn.execute(
		"
			INSERT INTO todos
			(
				id,
				label
			) VALUES (
				?1,
				?2
			)
		",
		params![todo.id, todo.label],
	) {
		Ok(_) => Ok(()),
		Err(e) => {
			panic!("Error: {}", e);
		},
	}
}

fn insert_todo_values(start_count: i64, sync_db_name: &str, conn: &Connection) -> Result<()> {
	let mut count = start_count;

	loop {
		let name = format!("TODO-{}", count);

		let result = add_todo(
			&Todo {
				id: count,
				label: name.clone(),
			},
			conn,
		);

		match result {
			Ok(_) => println!("Added todo: {}", name),
			Err(e) => panic!("Error: {}", e),
		}

		count += 1;

		if (count % 5) == 0 {
			sync(sync_db_name);
		}

		std::thread::sleep(std::time::Duration::from_secs(5));
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Example {
	table: String,
	pk: Vec<u8>,
	cid: String,
	val: Val,
	col_version: i64,
	db_version: i64,
	site_id: Vec<u8>,
	cl: i64,
	seq: i64,
}

fn sync(db_name: &str) {
	let conn = match get_db_connection(&String::from(db_name)) {
		Ok(conn) => conn,
		Err(e) => panic!("Error: {}", e),
	};

	let mut stmt = match conn.prepare("SELECT * FROM crsql_changes") {
		Ok(stmt) => stmt,
		Err(e) => panic!("Error: {}", e),
	};

	let rows_iter = from_rows::<Example>(stmt.query([]).unwrap());

	let changes = rows_iter.collect::<Vec<_>>();

	println!("Changes: {:?}", changes);
}
