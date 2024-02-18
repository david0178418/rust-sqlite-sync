use core::panic;
use rusqlite::{params, Connection, Result};
use serde_derive::{Deserialize, Serialize};

pub fn get_db_connection(db_name: &String) -> Result<Connection> {
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

pub fn fetch_table_max_id(table_name: &str, conn: &Connection) -> Result<i64> {
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

pub struct Todo {
	pub id: i64,
	pub label: String,
}

pub fn add_todo(todo: &Todo, conn: &Connection) -> Result<()> {
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DbSyncInfo {
	pub db_version: i64,
	pub site_id: String,
}

pub fn fetch_db_info(conn: &Connection) -> Result<DbSyncInfo> {
	let mut stmt = match conn.prepare(
		"
			SELECT
				crsql_db_version() as db_version,
				Hex(crsql_site_id()) as site_id;
	",
	) {
		Ok(stmt) => stmt,
		Err(e) => panic!("Error: {}", e),
	};

	let result = stmt.query_row([], |row| {
		Ok(DbSyncInfo {
			db_version: row.get(0)?,
			site_id: row.get(1)?,
		})
	});

	match result {
		Ok(info) => Ok(info),
		Err(e) => panic!("Error: {}", e),
	}
}
