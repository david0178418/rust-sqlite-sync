-- BEGIN TRANSACTION;

-- 	CREATE TABLE IF NOT EXISTS todo_lists (
-- 		id INTEGER PRIMARY KEY NOT NULL DEFAULT 0,
-- 		name TEXT NOT NULL DEFAULT 'New List'
-- 	);

-- 	CREATE TABLE IF NOT EXISTS todos (
-- 		id INTEGER PRIMARY KEY NOT NULL DEFAULT 0,
-- 		todo_lists_id INTEGER NOT NULL DEFAULT 0,
-- 		label TEXT NOT NULL DEFAULT 'New Todo',
-- 		completed INTEGER DEFAULT 0
-- 	);

-- 	SELECT crsql_as_crr('todo_lists');
-- 	SELECT crsql_as_crr('todos');

-- COMMIT;

BEGIN TRANSACTION;

	CREATE TABLE IF NOT EXISTS todos (
		id INTEGER PRIMARY KEY NOT NULL DEFAULT 0,
		label TEXT NOT NULL DEFAULT 'New Todo',
		completed INTEGER DEFAULT 0
	);

	SELECT crsql_as_crr('todos');

COMMIT;
