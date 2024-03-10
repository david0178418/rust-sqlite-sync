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

// TODO Extract tests to a separate file
#[cfg(test)]
mod tests {
	use crate::queries::{Foo, NewTodo, Todo};

	#[test]
	fn test_sync_new_items_a_to_b() {
		let foo_a = Foo::new(None).unwrap();
		let mut foo_b = Foo::new(None).unwrap();

		foo_a
			.insert_todo(&NewTodo {
				label: String::from("Test A1"),
			})
			.unwrap();

		let todos_a = foo_a.fetch_todos().unwrap();
		let todos_b = foo_b.fetch_todos().unwrap();

		assert_eq!(todos_a.len(), 1);
		assert_eq!(todos_b.len(), 0);

		let changes_a = foo_a.fetch_db_changes().unwrap();

		foo_b.insert_db_changes(&changes_a).unwrap();

		let todos_b = foo_b.fetch_todos().unwrap();

		assert_eq!(todos_b.len(), 1);
	}

	#[test]
	fn test_sync_new_items_a_b_c() {
		let mut foo_a = Foo::new(None).unwrap();
		let mut foo_b = Foo::new(None).unwrap();
		let mut foo_c = Foo::new(None).unwrap();

		foo_a
			.insert_todo(&NewTodo {
				label: String::from("Test A1"),
			})
			.unwrap();

		foo_b
			.insert_todo(&NewTodo {
				label: String::from("Test B1"),
			})
			.unwrap();

		let changes_a_to_b = foo_a.fetch_db_changes().unwrap();
		let changes_a_to_c = foo_a.fetch_db_changes().unwrap();
		let changes_b_to_a = foo_b.fetch_db_changes().unwrap();
		let changes_b_to_c = foo_b.fetch_db_changes().unwrap();

		foo_a.insert_db_changes(&changes_b_to_a).unwrap();
		foo_b.insert_db_changes(&changes_a_to_b).unwrap();
		foo_c.insert_db_changes(&changes_a_to_c).unwrap();
		foo_c.insert_db_changes(&changes_b_to_c).unwrap();

		let todos_a = foo_a.fetch_todos().unwrap();
		let todos_b = foo_b.fetch_todos().unwrap();
		let todos_c = foo_c.fetch_todos().unwrap();

		assert_eq!(todos_a.len(), 2);
		assert_eq!(todos_b.len(), 2);
		assert_eq!(todos_c.len(), 2);
	}

	#[test]
	fn test_sync_a_b_c() {
		let mut foo_a = Foo::new(None).unwrap();
		let mut foo_b = Foo::new(None).unwrap();
		let mut foo_c = Foo::new(None).unwrap();

		foo_a
			.insert_todo(&NewTodo {
				label: String::from("Test A1"),
			})
			.unwrap();

		foo_b
			.insert_todo(&NewTodo {
				label: String::from("Test B1"),
			})
			.unwrap();

		let changes_a_to_b = foo_a.fetch_db_changes().unwrap();
		let changes_a_to_c = foo_a.fetch_db_changes().unwrap();
		let changes_b_to_a = foo_b.fetch_db_changes().unwrap();
		let changes_b_to_c = foo_b.fetch_db_changes().unwrap();

		foo_a.insert_db_changes(&changes_b_to_a).unwrap();
		foo_b.insert_db_changes(&changes_a_to_b).unwrap();
		foo_c.insert_db_changes(&changes_a_to_c).unwrap();
		foo_c.insert_db_changes(&changes_b_to_c).unwrap();

		let todos_a = foo_a.fetch_todos().unwrap();
		let todos_b = foo_b.fetch_todos().unwrap();
		let todos_c = foo_c.fetch_todos().unwrap();

		assert_eq!(todos_a.len(), 2);
		assert_eq!(todos_b.len(), 2);
		assert_eq!(todos_c.len(), 2);
	}

	#[test]
	fn test_sync_update() {
		let mut foo_a = Foo::new(None).unwrap();
		let mut foo_b = Foo::new(None).unwrap();

		let todo_id = foo_a
			.insert_todo(&NewTodo {
				label: String::from("Test A1"),
			})
			.unwrap();

		let changes_a_to_b = foo_a.fetch_db_changes().unwrap();

		foo_b.insert_db_changes(&changes_a_to_b).unwrap();

		foo_a
			.update_todo(&Todo {
				id: todo_id.clone(),
				label: String::from("Test A1 Updated in A Again"),
			})
			.unwrap();

		let changes_a_to_b = foo_a.fetch_db_changes().unwrap();
		let changes_b_to_a = foo_b.fetch_db_changes().unwrap();

		foo_b.insert_db_changes(&changes_a_to_b).unwrap();
		foo_a.insert_db_changes(&changes_b_to_a).unwrap();

		let todo_a = foo_a.fetch_todo_by_id(&todo_id).unwrap();
		let todo_b = foo_b.fetch_todo_by_id(&todo_id).unwrap();

		assert_eq!(todo_a.label, "Test A1 Updated in A Again");
		assert_eq!(todo_b.label, "Test A1 Updated in A Again");
	}

	#[test]
	fn test_sync_update_conflicting() {
		let mut foo_a = Foo::new(None).unwrap();
		let mut foo_b = Foo::new(None).unwrap();

		let foo_id = foo_a
			.insert_todo(&NewTodo {
				label: String::from("Test A1"),
			})
			.unwrap();

		let changes_a_to_b = foo_a.fetch_db_changes().unwrap();

		foo_b.insert_db_changes(&changes_a_to_b).unwrap();

		foo_a
			.update_todo(&Todo {
				id: foo_id.clone(),
				label: String::from("Test A1 Updated in A Again"),
			})
			.unwrap();

		foo_b
			.update_todo(&Todo {
				id: foo_id.clone(),
				label: String::from("Test A1 Updated in B"),
			})
			.unwrap();

		let changes_a_to_b = foo_a.fetch_db_changes().unwrap();
		let changes_b_to_a = foo_b.fetch_db_changes().unwrap();

		foo_b.insert_db_changes(&changes_a_to_b).unwrap();
		foo_a.insert_db_changes(&changes_b_to_a).unwrap();

		let todo_a = foo_a.fetch_todo_by_id(&foo_id).unwrap();
		let todo_b = foo_b.fetch_todo_by_id(&foo_id).unwrap();

		assert_eq!(todo_a.label, "Test A1 Updated in B");
		assert_eq!(todo_b.label, "Test A1 Updated in B");
	}
}
