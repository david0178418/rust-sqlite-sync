mod queries;

use crate::queries::{fetch_db_info, insert_changes};
use core::panic;
use queries::{add_todo, fetch_table_max_id, get_db_connection, Change, Todo};
use rusqlite::{named_params, Result};
use serde_rusqlite::from_rows;
use std::env;

fn main() -> Result<()> {
	let db_name = &parse_db_name(env::args().collect());

	insert_todo_values(db_name)
}

fn parse_db_name(args: Vec<String>) -> String {
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

fn insert_todo_values(sync_db_name: &str) -> Result<()> {
	let conn = match get_db_connection(&String::from(sync_db_name)) {
		Ok(conn) => conn,
		Err(e) => panic!("Error: {}", e),
	};

	let max_id = match fetch_table_max_id("todos", &conn) {
		Ok(id) => id,
		Err(e) => panic!("Error: {}", e),
	};

	println!("Max ID: {}", max_id);

	let mut count = max_id + 1;

	loop {
		std::thread::sleep(std::time::Duration::from_secs(5));

		let name = format!("TODO-{}", count);

		let result = add_todo(
			&Todo {
				id: count,
				label: name.clone(),
			},
			&conn,
		);

		match result {
			Ok(_) => println!("Added todo: {}", name),
			Err(e) => panic!("Error: {}", e),
		}

		count += 1;

		if (count % 2) == 0 {
			sync(sync_db_name, &sync_db_name.replace(".db", "-sync.db"));
		}
	}
}

fn sync(source_db_name: &str, target_db_name: &str) {
	println!("Syncing from {} to {}", source_db_name, target_db_name);
	let source_db_connection = match get_db_connection(&String::from(source_db_name)) {
		Ok(conn) => conn,
		Err(e) => panic!("Error: {}", e),
	};

	let mut target_db_connection = match get_db_connection(&String::from(target_db_name)) {
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
			pk,
			cid,
			val,
			col_version,
			db_version,
			COALESCE(
				site_id,
				crsql_site_id()
			) as site_id,
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

	let rows_iter = from_rows::<Change>(result);

	// let changes = rows_iter.collect::<Vec<_>>();

	let changes = rows_iter
		.map(|change| match change {
			Ok(c) => c,
			Err(e) => panic!("Error: {}", e),
		})
		.collect::<Vec<_>>();

	insert_changes(&changes, &mut target_db_connection);

	println!(
		"Synced {} rows from {} to {}",
		changes.len(),
		source_db_name,
		target_db_name
	);
}
