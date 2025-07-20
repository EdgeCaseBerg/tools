use std::fs;
use rusqlite::{Connection, Result};
use std::path::{self, PathBuf };

const DATABASE_FILE: &str = "helloworld.db";

pub fn connect_to_sqlite() -> Result<Connection, rusqlite::Error> {
    Connection::open(DATABASE_FILE)
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
DELETE FROM dupdb_filehashes WHERE hash = ?1 AND file_path = ?2
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

#[cfg(test)]
mod tests {
    use super::*;
 
    static mut TEST_DB_NO: u32 = 0;

    fn open_test_database() -> Connection {
        let connection;

        unsafe {
            TEST_DB_NO += 1;
            let filename = format!("test_{TEST_DB_NO}.sqlite.db");
            let _ = fs::remove_file(&filename);
            connection = Connection::open(filename).expect("Cannot open database for test");
        };
        initialize(&connection);
        connection
    }

    #[test]
    fn count_of_non_existing_hash_should_be_0() {
        let connection = open_test_database();
        let count_of_nothingness = count_of_same_hash(&connection, 196248234750);
        let marking_time_waiting_for_death = 0;
        assert_eq!(count_of_nothingness, marking_time_waiting_for_death);
    }

    #[test]
    fn count_of_existing_hash_should_be_n() {
        let connection = open_test_database();
        let hash = 123456789;
        let path = "12345689";
        // Insert something other than the one we're testing too
        insert_file_hash(&connection, 9876543211, "987654321");
        insert_file_hash(&connection, hash, path);
        let begin_instrumentality = count_of_same_hash(&connection, hash);
        let hall_of_goff = 1;
        assert_eq!(begin_instrumentality, hall_of_goff);
        insert_file_hash(&connection, hash, path);
        let rejoicing_of_the_masses = count_of_same_hash(&connection, hash);
        assert_eq!(rejoicing_of_the_masses, 2);
    }

    #[test]
    fn select_dupes_based_on_filepath_hash() {
        let connection = open_test_database();
        // Insert something other than the one we're testing too
        insert_file_hash(&connection, 9876543211, "987654321");
        let hash = 1234567;
        let path = "hellothere";
        for i in 0..10 {
            insert_file_hash(&connection, hash, path);
        }
        let there_should_be_10_dupes = dups_by_file(&connection, path);
        assert_eq!(there_should_be_10_dupes.len(), 10);
        for (db_hash, db_path) in there_should_be_10_dupes {
            assert_eq!(hash, db_hash.parse::<u64>().expect("could not parse db u64 string to u64"));
            assert_eq!(path, db_path);
        }
    }

    #[test]
    fn can_delete_from_database_for_matches() {
        let connection = open_test_database();
        // Insert something other than the one we're testing too
        insert_file_hash(&connection, 9876543211, "987654321");
        let hash = 1234567;
        let path = "hellothere";
        for i in 0..2 {
            insert_file_hash(&connection, hash, path);
        }
        let there_should_be_2_dupes = dups_by_file(&connection, path);
        assert_eq!(there_should_be_2_dupes.len(), 2);
        let deleted = delete_all_matching(&connection, hash, path);
        assert_eq!(deleted, 2);
        let should_be_zero = count_of_same_hash(&connection, hash);
        assert_eq!(should_be_zero, 0);
    }
}