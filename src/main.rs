use core::panic;
use std::env;
use rusqlite::{Connection, Result, params};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_db_name = String::from("test.db");
    let db_name = args.get(1).unwrap_or(&input_db_name);
    let db_name = if db_name.ends_with(".db") {
        db_name.to_string()
    } else {
        format!("{}.db", db_name)
    };

    let conn = Connection::open(db_name)?;

    println!("Loading extension... ");

    unsafe {
        conn.load_extension_enable()?;
        conn.load_extension("./crsqlite.so", Some("sqlite3_crsqlite_init"))?;
        conn.load_extension_disable()?;
    }

    println!("Done loading extension");

    // load and execute init.sql file
    let init_sql = include_str!("init.sql");

    conn.execute_batch(init_sql)?;


    let mut stmt = conn.prepare("SELECT MAX(ID) as MAX FROM todos;")?;
    let mut rows = stmt.query([])?;

    let row = match rows.next() {
        Ok(Some(row)) => row,
        Ok(None) => {
            panic!("Error: no rows found");
        }
        Err(e) => {
            panic!("Error: no rows found: {}", e);
        }
    };

    let max_id = match row.get::<_, i64>(0) {
        Ok(id) => id,
        Err(e) => {
            println!("Error: max_id{}", e);
            0
        }
    };

    println!("Max ID: {}", max_id);

    // insert some data on a 5 second interval until the program is killed
    let mut count = max_id;

    loop {
        count += 1;
        let name = format!("TODO-{}", count);

        conn.execute(
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
            params![&count, &name],
        )?;

        println!("Inserted '{}'", name);

        std::thread::sleep(std::time::Duration::from_secs(5));
    }

}
