use core::panic;
use rusqlite::{named_params, params, Connection, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use serde_rusqlite::from_rows;

pub fn get_db_connection(db_name: &String) -> Result<Connection> {
	println!("Opening connection to {}", db_name);
	let conn = Connection::open(db_name)?;

	println!("Loading extension... ");

	load_cr_extention(&conn)?;

	conn.execute_batch(include_str!("init.sql"))?;

	Ok(conn)
}

pub fn load_cr_extention(conn: &Connection) -> Result<()> {
	unsafe {
		conn.load_extension_enable()?;
		conn.load_extension("./crsqlite.so", Some("sqlite3_crsqlite_init"))?;
		conn.load_extension_disable()?;
	}

	Ok(())
}

pub fn fetch_table_max_id(table_name: &str, conn: &Connection) -> Result<i64> {
	let mut stmt = conn.prepare(&format!("SELECT MAX(ID) as MAX FROM {};", table_name))?;
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

#[allow(unused)]
pub fn fetch_todos(conn: &Connection) -> Result<Vec<Todo>> {
	let mut stmt = conn.prepare("SELECT * FROM todos;")?;
	let todos = stmt.query_map([], |row| {
		Ok(Todo {
			id: row.get(0)?,
			label: row.get(1)?,
		})
	})?;

	let mut todo_list = Vec::new();

	for todo in todos {
		todo_list.push(todo?);
	}

	Ok(todo_list)
}

#[allow(unused)]
pub fn fetch_todo_by_id(id: i64, conn: &Connection) -> Result<Todo> {
	conn.query_row("SELECT * FROM todos WHERE id = ?1", params![id], |row| {
		Ok(Todo {
			id: row.get(0)?,
			label: row.get(1)?,
		})
	})
}

pub struct Todo {
	pub id: i64,
	pub label: String,
}

pub fn insert_todo(todo: &Todo, conn: &Connection) -> Result<()> {
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
		Err(e) => panic!("Error: {}", e),
	}
}

#[allow(unused)]
pub fn update_todo(todo: &Todo, conn: &Connection) -> Result<()> {
	match conn.execute(
		"
			UPDATE todos
			SET label = ?2
			WHERE id = ?1
		",
		params![todo.id, todo.label],
	) {
		Ok(_) => Ok(()),
		Err(e) => panic!("Error: {}", e),
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DbSyncInfo {
	pub db_version: i64,
	pub site_id: String,
}

pub fn fetch_db_info(conn: &Connection) -> Result<DbSyncInfo> {
	conn.prepare(
		"
		SELECT
			COALESCE(
				(
					select max(db_version)
					from crsql_changes
					where site_id is null
				),
				0
			) AS db_version,
			HEX(crsql_site_id()) AS site_id;
		",
	)?
	.query_row([], |row| {
		Ok(DbSyncInfo {
			db_version: row.get(0)?,
			site_id: row.get(1)?,
		})
	})
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Change {
	table: String,
	pk: Vec<u8>,
	cid: String,
	val: Value,
	col_version: i64,
	db_version: i64,
	site_id: Vec<u8>,
	cl: i64,
	seq: i64,
}

pub fn insert_db_changes(changes: &Vec<Change>, conn: &mut Connection) -> Result<()> {
	let tx = conn.transaction()?;

	for change in changes {
		let x = serde_json::to_string(&change.val).unwrap();
		println!(
			"Inserting change: {:?}",
			match change.val.is_string() {
				true => change.val.as_str().unwrap(),
				false => &x,
			}
		);
		let change_val = serde_json::to_string(&change.val).unwrap();
		let result = tx.execute(
			"
				INSERT INTO crsql_changes
				(
					\"table\",
					pk,
					cid,
					val,
					col_version,
					db_version,
					site_id,
					cl,
					seq
				) VALUES (
					:table,
					:pk,
					:cid,
					:val,
					:col_version,
					:db_version,
					:site_id,
					:cl,
					:seq
				)
			",
			named_params! {
				":table": change.table,
				":pk": change.pk,
				":cid": change.cid,
				":col_version": change.col_version,
				":db_version": change.db_version,
				":site_id": change.site_id,
				":cl": change.cl,
				":seq": change.seq,
				":val": if change.val.is_string() {
					change.val.as_str().unwrap()
				} else{
					&change_val
				}
			},
		);

		match result {
			Ok(_) => (),
			Err(e) => panic!("Error: {}", e),
		}
	}

	tx.commit()
}

pub fn fetch_db_changes(db_sync_info: &DbSyncInfo, conn: &Connection) -> Result<Vec<Change>> {
	println!("Fetching changes for {:?}", db_sync_info);
	let mut stmt = conn.prepare(
		"
		SELECT
			\"table\",
			pk,
			cid,
			val,
			col_version,
			db_version,
			site_id,
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

	Ok(changes)
}

// TODO Extract tests to a separate file
#[cfg(test)]
mod tests {
	use crate::queries::{
		fetch_db_changes, fetch_db_info, fetch_todo_by_id, fetch_todos, insert_db_changes,
		insert_todo, load_cr_extention, update_todo, Todo,
	};

	fn setup_connection() -> rusqlite::Connection {
		let conn = rusqlite::Connection::open_in_memory().unwrap();

		load_cr_extention(&conn).unwrap();
		conn.execute_batch(include_str!("init.sql")).unwrap();

		conn
	}

	#[test]
	fn test_sync_new_items_a_to_b() {
		let conn_a = setup_connection();
		let mut conn_b = setup_connection();

		insert_todo(
			&Todo {
				id: 1,
				label: String::from("Test A1"),
			},
			&conn_a,
		)
		.unwrap();

		let todos_a = fetch_todos(&conn_a).unwrap();
		let todos_b = fetch_todos(&conn_b).unwrap();

		assert_eq!(todos_a.len(), 1);
		assert_eq!(todos_b.len(), 0);

		let db_b_sync_info = fetch_db_info(&conn_b).unwrap();
		let changes_a = fetch_db_changes(&db_b_sync_info, &conn_a).unwrap();

		insert_db_changes(&changes_a, &mut conn_b).unwrap();

		let todos_b = fetch_todos(&conn_b).unwrap();

		assert_eq!(todos_b.len(), 1);
	}

	#[test]
	fn test_sync_new_items_a_b_c() {
		let mut conn_a = setup_connection();
		let mut conn_b = setup_connection();
		let mut conn_c = setup_connection();

		insert_todo(
			&Todo {
				id: 1,
				label: String::from("Test A1"),
			},
			&conn_a,
		)
		.unwrap();

		insert_todo(
			&Todo {
				id: 2,
				label: String::from("Test B1"),
			},
			&conn_b,
		)
		.unwrap();

		let db_a_sync_info = fetch_db_info(&conn_a).unwrap();
		let db_b_sync_info = fetch_db_info(&conn_b).unwrap();
		let db_c_sync_info = fetch_db_info(&conn_c).unwrap();

		let changes_a_to_b = fetch_db_changes(&db_b_sync_info, &conn_a).unwrap();
		let changes_a_to_c = fetch_db_changes(&db_c_sync_info, &conn_a).unwrap();
		let changes_b_to_a = fetch_db_changes(&db_a_sync_info, &conn_b).unwrap();
		let changes_b_to_c = fetch_db_changes(&db_c_sync_info, &conn_b).unwrap();

		insert_db_changes(&changes_b_to_a, &mut conn_a).unwrap();
		insert_db_changes(&changes_a_to_b, &mut conn_b).unwrap();
		insert_db_changes(&changes_a_to_c, &mut conn_c).unwrap();
		insert_db_changes(&changes_b_to_c, &mut conn_c).unwrap();

		let todos_a = fetch_todos(&conn_a).unwrap();
		let todos_b = fetch_todos(&conn_b).unwrap();
		let todos_c = fetch_todos(&conn_c).unwrap();

		assert_eq!(todos_a.len(), 2);
		assert_eq!(todos_b.len(), 2);
		assert_eq!(todos_c.len(), 2);
	}

	#[test]
	fn test_sync_a_b_c() {
		let mut conn_a = setup_connection();
		let mut conn_b = setup_connection();
		let mut conn_c = setup_connection();

		insert_todo(
			&Todo {
				id: 1,
				label: String::from("Test A1"),
			},
			&conn_a,
		)
		.unwrap();

		insert_todo(
			&Todo {
				id: 2,
				label: String::from("Test B1"),
			},
			&conn_b,
		)
		.unwrap();

		let db_a_sync_info = fetch_db_info(&conn_a).unwrap();
		let db_b_sync_info = fetch_db_info(&conn_b).unwrap();
		let db_c_sync_info = fetch_db_info(&conn_c).unwrap();

		let changes_a_to_b = fetch_db_changes(&db_b_sync_info, &conn_a).unwrap();
		let changes_a_to_c = fetch_db_changes(&db_c_sync_info, &conn_a).unwrap();
		let changes_b_to_a = fetch_db_changes(&db_a_sync_info, &conn_b).unwrap();
		let changes_b_to_c = fetch_db_changes(&db_c_sync_info, &conn_b).unwrap();

		insert_db_changes(&changes_b_to_a, &mut conn_a).unwrap();
		insert_db_changes(&changes_a_to_b, &mut conn_b).unwrap();
		insert_db_changes(&changes_a_to_c, &mut conn_c).unwrap();
		insert_db_changes(&changes_b_to_c, &mut conn_c).unwrap();

		let todos_a = fetch_todos(&conn_a).unwrap();
		let todos_b = fetch_todos(&conn_b).unwrap();
		let todos_c = fetch_todos(&conn_c).unwrap();

		assert_eq!(todos_a.len(), 2);
		assert_eq!(todos_b.len(), 2);
		assert_eq!(todos_c.len(), 2);
	}

	#[test]
	fn test_sync_update() {
		let mut conn_a = setup_connection();
		let mut conn_b = setup_connection();

		insert_todo(
			&Todo {
				id: 1,
				label: String::from("Test A1"),
			},
			&conn_a,
		)
		.unwrap();

		let db_b_sync_info = fetch_db_info(&conn_b).unwrap();

		let changes_a_to_b = fetch_db_changes(&db_b_sync_info, &conn_a).unwrap();

		insert_db_changes(&changes_a_to_b, &mut conn_b).unwrap();

		update_todo(
			&Todo {
				id: 1,
				label: String::from("Test A1 Updated in A Again"),
			},
			&conn_a,
		)
		.unwrap();

		let db_a_sync_info = fetch_db_info(&conn_a).unwrap();
		let db_b_sync_info = fetch_db_info(&conn_b).unwrap();

		let changes_a_to_b = fetch_db_changes(&db_b_sync_info, &conn_a).unwrap();
		let changes_b_to_a = fetch_db_changes(&db_a_sync_info, &conn_b).unwrap();

		insert_db_changes(&changes_a_to_b, &mut conn_b).unwrap();
		insert_db_changes(&changes_b_to_a, &mut conn_a).unwrap();

		let todo_a = fetch_todo_by_id(1, &conn_a).unwrap();
		let todo_b = fetch_todo_by_id(1, &conn_a).unwrap();

		assert_eq!(todo_a.label, "Test A1 Updated in A Again");
		assert_eq!(todo_b.label, "Test A1 Updated in A Again");
	}

	#[test]
	fn test_sync_update_conflicting() {
		let mut conn_a = setup_connection();
		let mut conn_b = setup_connection();

		insert_todo(
			&Todo {
				id: 1,
				label: String::from("Test A1"),
			},
			&conn_a,
		)
		.unwrap();

		let db_b_sync_info = fetch_db_info(&conn_b).unwrap();

		let changes_a_to_b = fetch_db_changes(&db_b_sync_info, &conn_a).unwrap();

		insert_db_changes(&changes_a_to_b, &mut conn_b).unwrap();

		update_todo(
			&Todo {
				id: 1,
				label: String::from("Test A1 Updated in A Again"),
			},
			&conn_a,
		)
		.unwrap();

		update_todo(
			&Todo {
				id: 1,
				label: String::from("Test A1 Updated in B"),
			},
			&conn_b,
		)
		.unwrap();

		let db_a_sync_info = fetch_db_info(&conn_a).unwrap();
		let db_b_sync_info = fetch_db_info(&conn_b).unwrap();

		let changes_a_to_b = fetch_db_changes(&db_b_sync_info, &conn_a).unwrap();
		let changes_b_to_a = fetch_db_changes(&db_a_sync_info, &conn_b).unwrap();

		insert_db_changes(&changes_a_to_b, &mut conn_b).unwrap();
		insert_db_changes(&changes_b_to_a, &mut conn_a).unwrap();

		let todo_a = fetch_todo_by_id(1, &conn_a).unwrap();
		let todo_b = fetch_todo_by_id(1, &conn_a).unwrap();

		assert_eq!(todo_a.label, "Test A1 Updated in B");
		assert_eq!(todo_b.label, "Test A1 Updated in B");
	}
}
