use std::fs;
use rusqlite::{Connection, Result};
use std::path::{self, PathBuf };

const DATABASE_FILE: &str = "helloworld.db";

pub fn connect_to_sqlite() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(DATABASE_FILE);
    conn
}

const SQL_CREATE_TABLE: &str = "
CREATE TABLE IF NOT EXISTS dupdb_filehashes (
    hash TEXT NOT NULL,
    file_path TEXT NOT NULL
)";

const SQL_CREATE_INDICES: &str = "
CREATE INDEX IF NOT EXISTS hash_index ON dupdb_filehashes (hash);
";

pub fn initialize(sqlite_connection: &Connection) {
	sqlite_connection.execute(SQL_CREATE_TABLE, ()).expect("Could not create sqlite table");
    sqlite_connection.execute(SQL_CREATE_INDICES, ()).expect("Could not setup indices on sqlite db");
}

const SQL_INSERT_HASH_AND_FILEPATH: &str = "
INSERT INTO dupdb_filehashes (hash, file_path) VALUES (?1, ?2)
";

pub fn insert_file_hash(conn: &Connection, hash: u64, absolute_path: &str) -> bool {
	let mut statement = conn.prepare_cached(SQL_INSERT_HASH_AND_FILEPATH).expect("could not prepare insertion statement");
    match statement.execute((hash.to_string(), absolute_path)) {
    	Ok(rows_inserted) => rows_inserted == 1,
    	Err(err) => {
    		eprintln!("Unable to insert into table failed: {}", err);
    		false
    	}
    }
}

const SQL_SELECT_COUNT_FOR_HASH: &str = "
SELECT COUNT(*) FROM dupdb_filehashes WHERE hash = ?1
";

pub fn count_of_same_hash(conn: &Connection, hash: u64) -> u32 {
	let mut statement = conn.prepare_cached(SQL_SELECT_COUNT_FOR_HASH)
		.expect("Could not fetch prepared count query");

	match statement.query_one([hash.to_string()], |row| row.get::<_, u32>(0)) {
		Err(err) => {
    		eprintln!("Unable to count rows in table: {}", err);
    		0
    	},
		Ok(count) => count
	}
}

const SQL_SELECT_DUPES_FOR_FILE: &str = "
SELECT hash, file_path FROM dupdb_filehashes
	WHERE hash IN (SELECT hash FROM dupdb_filehashes WHERE file_path = ?1)
";

pub fn dups_by_file(conn: &Connection, absolute_path: &str) -> Vec<(String, String)> {
	let mut statement = conn.prepare_cached(SQL_SELECT_DUPES_FOR_FILE)
		.expect("Could not fetch prepared select_dups query");

	let rows = statement.query_map([absolute_path], |row| {
        Ok((
            row.get::<usize, String>(0).expect("could not retrieve hash column 0 for select row"), 
            row.get::<usize, String>(1).expect("could not retrieve file_path column 1 for select row")
        ))
    });

	let mut dups = Vec::new();
	match rows {
		Err(binding_failure) => {
			eprintln!("Unable to select rows from table: {}", binding_failure);
		},
		Ok(mapped_rows) => {
			for result in mapped_rows {
				let tuple = result
					.expect("Impossible. Expect should have failed in query_map before this ever occured");
				dups.push(tuple);
			}
		}
    }

	dups
}

const SQL_DELETE_BY_HASH_AND_FILE: &str ="
DELETE FROM dupdb_filehashes WHERE rowid = (
	SELECT rowid FROM dupdb_filehashes WHERE hash = ?1 AND file_path = ?2 LIMIT 1
)
";

pub fn delete_all_matching(conn: &Connection, hash: u64, absolute_path: &str) -> usize {
    let mut statement = conn.prepare_cached(SQL_DELETE_BY_HASH_AND_FILE)
    	.expect("Failed to prepare delete statement");

    match statement.execute([hash.to_string(), absolute_path.to_string()]) {
    	Ok(rows_deleted) => rows_deleted,
    	Err(err) => {
    		eprintln!("Unable to delete from dupdb_filehashes: {}", err);
    		0
    	}
    }
}

const SQL_DROP_TABLE: &str = "
DROP TABLE dupdb_filehashes
";

pub fn reset_all_data(sqlite_connection: &Connection) {
	sqlite_connection.execute(SQL_DROP_TABLE, ())
		.expect("Could not drop database. Go delete it yourself.");

	initialize(sqlite_connection);
}


pub fn add_hash_to_db_test(conn: &Connection, file_path: PathBuf) {
    let bytes = fs::read(&file_path).expect("could not read path");
    let hash = seahash::hash(&bytes);
    let absolute_path = path::absolute(file_path)
        .expect("Unable to get absolute path for file to hash").to_str()
        .expect("Unexpected file name containining non utf 8 characters found").to_string();

    let success = insert_file_hash(conn, hash, &absolute_path);
    println!("success: {}", success);

    let mut statement = conn.prepare_cached("SELECT hash, file_path FROM dupdb_filehashes").expect("could not prepare select from file");
    let row_iter = statement.query_map([], |row| {
        Ok((
            row.get::<usize, String>(0).expect("could not retrieve column 0 for select row"), 
            row.get::<usize, String>(1).expect("could not retrieve column 1 for select row")
        ))
    }).expect("failed to query dupdb_filehashes table");
    for row in row_iter {
        println!("{:?}", row);
    }

    let count = count_of_same_hash(conn, hash);
    println!("Rows inserted so far {:?}", count);

    let dups = dups_by_file(conn, &absolute_path);
    println!("dups {:?}", dups);

    let how_many_removed = delete_all_matching(conn, hash, &absolute_path);
    println!("removed {:?}", how_many_removed);

    reset_all_data(conn);
}
