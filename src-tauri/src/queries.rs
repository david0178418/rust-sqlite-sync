#[cfg(test)]
#[path = "./queries_tests.rs"]
mod queries_tests;

use core::panic;
use rusqlite::{named_params, params, Connection, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use serde_rusqlite::from_rows;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NewTodo {
	pub label: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Todo {
	pub id: String,
	pub label: String,
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

pub struct Foo {
	connection: Connection,
}

impl Foo {
	#[allow(unused)]
	pub fn new(connection_string: Option<String>) -> Result<Self> {
		let connection = match connection_string {
			Some(connection_string) => Connection::open(connection_string).unwrap(),
			None => Connection::open_in_memory().unwrap(),
		};

		unsafe {
			connection.load_extension_enable()?;
			connection.load_extension("./crsqlite.so", Some("sqlite3_crsqlite_init"))?;
			connection.load_extension_disable()?;
		}

		connection.execute_batch(include_str!("init.sql"))?;

		Ok(Foo { connection })
	}

	#[allow(unused)]
	pub fn fetch_table_max_id(&self, table_name: &str) -> Result<i64> {
		let mut stmt = self
			.connection
			.prepare(&format!("SELECT MAX(ID) as MAX FROM {};", table_name))?;
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
	pub fn fetch_todos(&self) -> Result<Vec<Todo>> {
		let mut stmt = self.connection.prepare("SELECT * FROM todos;")?;
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
	pub fn fetch_todo_by_id(&self, id: &String) -> Result<Todo> {
		self.connection
			.query_row("SELECT * FROM todos WHERE id = ?1", params![id], |row| {
				Ok(Todo {
					id: row.get(0)?,
					label: row.get(1)?,
				})
			})
	}

	#[allow(unused)]
	pub fn insert_todo(&self, todo: &NewTodo) -> Result<String> {
		let new_id = Uuid::now_v7().to_string();

		match self.connection.execute(
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
			params![new_id, todo.label],
		) {
			Ok(_) => Ok(new_id),
			Err(e) => panic!("Error: {}", e),
		}
	}

	#[allow(unused)]
	pub fn delete_todo(&self, id: &String) -> Result<()> {
		match self
			.connection
			.execute("DELETE FROM todos WHERE id = ?1", params![id])
		{
			Ok(_) => Ok(()),
			Err(e) => panic!("Error: {}", e),
		}
	}

	#[allow(unused)]
	pub fn update_todo(&self, todo: &Todo) -> Result<()> {
		match self.connection.execute(
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

	#[allow(unused)]
	pub fn insert_db_changes(&mut self, changes: &Vec<Change>) -> Result<()> {
		let tx = self.connection.transaction()?;

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
					);
					select crsql_finalize();
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

	#[allow(unused)]
	pub fn fetch_db_changes(&self) -> Result<Vec<Change>> {
		let mut stmt = self.connection.prepare(
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
			FROM crsql_changes;
		",
		)?;

		let result = stmt.query([])?;

		let changes = from_rows::<Change>(result)
			.map(|change| change.unwrap())
			.collect::<Vec<_>>();

		Ok(changes)
	}
}
