mod queries;

use crate::queries::{fetch_db_info, insert_changes};
use queries::{fetch_table_max_id, get_db_connection, insert_todo, Change, Todo};
use rusqlite::{named_params, Result};
use serde_rusqlite::from_rows;
use std::env;

fn main() -> Result<()> {
	let db_name = &parse_db_name(env::args().collect());

	insert_todo_values(db_name)
}

fn parse_db_name(args: Vec<String>) -> String {
	let default_name = String::from("test.db");
	let db_name = args.get(1).unwrap_or(&default_name);

	if db_name.ends_with(".db") {
		db_name.to_string()
	} else {
		format!("{}.db", db_name)
	}
}

fn insert_todo_values(sync_db_name: &str) -> Result<()> {
	let conn = get_db_connection(&String::from(sync_db_name))?;

	let max_id = fetch_table_max_id("todos", &conn)?;

	println!("Max ID: {}", max_id);

	let mut count = max_id + 1;

	loop {
		std::thread::sleep(std::time::Duration::from_secs(5));

		let name = format!("TODO-{}", count);

		insert_todo(
			&Todo {
				id: count,
				label: name.clone(),
			},
			&conn,
		)?;

		count += 1;

		if (count % 3) == 0 {
			match sync(sync_db_name, &sync_db_name.replace(".db", "-sync.db")) {
				Ok(_) => (),
				Err(e) => println!("Sync Failed: {}", e),
			};
		}
	}
}

fn sync(source_db_name: &str, target_db_name: &str) -> Result<()> {
	println!("Syncing from {} to {}", source_db_name, target_db_name);
	let source_db_connection = get_db_connection(&String::from(source_db_name))?;

	let mut target_db_connection = get_db_connection(&String::from(target_db_name))?;

	let db_sync_info = fetch_db_info(&target_db_connection)?;

	println!("DB Version: {:?}", db_sync_info);

	let mut stmt = source_db_connection.prepare(
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
	)?;

	let result = stmt.query(named_params! {
		":db_version": db_sync_info.db_version,
		":site_id": db_sync_info.site_id,
	})?;

	let changes = from_rows::<Change>(result)
		.map(|change| change.unwrap())
		.collect::<Vec<_>>();

	insert_changes(&changes, &mut target_db_connection)?;

	println!(
		"Synced {} rows from {} to {}",
		changes.len(),
		source_db_name,
		target_db_name
	);

	Ok(())
}
