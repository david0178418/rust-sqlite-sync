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
	fn test_sync_delete() {
		let foo_a = Foo::new(None).unwrap();
		let mut foo_b = Foo::new(None).unwrap();

		foo_a
			.insert_todo(&NewTodo {
				label: String::from("Test A1"),
			})
			.unwrap();

		let foo_id = foo_a
			.insert_todo(&NewTodo {
				label: String::from("Test A2"),
			})
			.unwrap();

		foo_a
			.insert_todo(&NewTodo {
				label: String::from("Test A2"),
			})
			.unwrap();

		let changes_a_to_b = foo_a.fetch_db_changes().unwrap();

		foo_b.insert_db_changes(&changes_a_to_b).unwrap();

		let todos_a = foo_a.fetch_todos().unwrap();
		let todos_b = foo_b.fetch_todos().unwrap();

		assert_eq!(todos_a.len(), 3);
		assert_eq!(todos_b.len(), 3);

		foo_a.delete_todo(&foo_id).unwrap();

		let changes_a_to_b = foo_a.fetch_db_changes().unwrap();

		foo_b.insert_db_changes(&changes_a_to_b).unwrap();

		let todos_a = foo_a.fetch_todos().unwrap();
		let todos_b = foo_b.fetch_todos().unwrap();

		assert_eq!(todos_a.len(), 2);
		assert_eq!(todos_b.len(), 2);
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
