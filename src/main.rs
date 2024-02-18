mod queries;

use core::panic;
use queries::{add_todo, fetch_table_max_id, get_db_connection, Todo};
use rusqlite::{named_params, Connection, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value as Val;
use serde_rusqlite::from_rows;
use std::env;

use crate::queries::fetch_db_info;

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

fn insert_todo_values(start_count: i64, sync_db_name: &str, conn: &Connection) -> Result<()> {
	let mut count = start_count;

	loop {
		std::thread::sleep(std::time::Duration::from_secs(5));

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
			sync(sync_db_name, &format!("{sync_db_name}-2"));
		}
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Example {
	table: String,
	pk: String,
	cid: String,
	val: Val,
	col_version: i64,
	db_version: i64,
	site_id: String,
	cl: i64,
	seq: i64,
}

fn sync(source_db_name: &str, target_db_name: &str) {
	let source_db_connection = match get_db_connection(&String::from(source_db_name)) {
		Ok(conn) => conn,
		Err(e) => panic!("Error: {}", e),
	};

	let target_db_connection = match get_db_connection(&String::from(target_db_name)) {
		Ok(conn) => conn,
		Err(e) => panic!("Error: {}", e),
	};

	let db_sync_info = match fetch_db_info(&target_db_connection) {
		Ok(version) => version,
		Err(e) => panic!("Error: {}", e),
	};

	println!("DB Version: {:?}", db_sync_info);

	let mut stmt = match source_db_connection.prepare(
		"
		SELECT
			\"table\",
			HEX(pk) as pk,
			cid,
			val,
			col_version,
			db_version,
			HEX(COALESCE(
				site_id,
				crsql_site_id()
			)) as site_id,
			cl,
			seq
		FROM crsql_changes
		WHERE db_version > :db_version
		AND site_id IS NOT :site_id;
	",
	) {
		Ok(stmt) => stmt,
		Err(e) => panic!("Error: {}", e),
	};

	let query = stmt.query(named_params! {
		":db_version": db_sync_info.db_version,
		":site_id": db_sync_info.site_id,
	});

	let result = match query {
		Ok(r) => r,
		Err(e) => panic!("Error: {}", e),
	};

	let rows_iter = from_rows::<Example>(result);

	let changes = rows_iter.collect::<Vec<_>>();

	println!("Changes: {:?}", changes);
}
