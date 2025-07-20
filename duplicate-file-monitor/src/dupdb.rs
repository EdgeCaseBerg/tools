pub use std::env;
use std::path::{self, Path, PathBuf };
use std::fs::{ self };
use std::time::Duration;

use notify::{self, RecursiveMode, EventKind};
use notify_debouncer_full::new_debouncer;

use std::sync::mpsc;

use notify_rust::Notification;

use nav_update::RecursiveDirIterator;
use rusqlite::Connection;

const NAME_OF_HIDDEN_FOLDER: &str = ".dupdb";
const NAME_OF_HASH_FILE: &str = "index.dat";
const APPNAME: &str = "Dup DB";
const DEBUGGING_LOCAL: bool = true;

use crate::sql;

#[derive(Debug)]
pub struct DuplicateDatabase {
    conn: Connection
}

impl DuplicateDatabase {
    pub fn add(&mut self, hash: u64, full_file_path: String) {
        let entered = sql::insert_file_hash(&self.conn, hash, &full_file_path);
        if !entered {
            eprintln!("Did not enter file path and hash into database: {}, {}", hash, full_file_path);
        }
    }

    pub fn contains_duplicate_for_hash(&self, hash: u64) -> bool {
        let count = sql::count_of_same_hash(&self.conn, hash);
        count > 1
    }

    pub fn remove(&mut self, full_file_path: String) {
        // TODO just replace with a plain DELETE query in sql and save the effort
        let references = sql::dups_by_file(&self.conn, &full_file_path);
        let to_remove: Vec<(String, String)> = references.into_iter().filter(|(_, file_path)| *file_path == full_file_path).collect();
        for (hash, filepath) in to_remove {
            let numeric_hash = hash.parse().expect("Hash stored in database was not parseable to u64");
            sql::delete_all_matching(&self.conn, numeric_hash, &filepath);
        }
    }

    pub fn debug_key(&self, full_file_path: String) {
        let references = sql::dups_by_file(&self.conn, &full_file_path);
        if references.len() == 0 {
            println!("Path {:?} not in files_to_hash list", full_file_path);
            return;
        }

        for (hash, file_path) in references {
            println!("Value: Hash: {:?} Path: {:?}", hash, file_path);
        }
    }
}

/// Returns true if new index was created, false otherwise
pub fn dupdb_initialize_hidden_folder() -> bool {
    let database_exists_already =dupdb_database_path().exists();
    if database_exists_already {
        return false;
    }
    let connection = sql::connect_to_sqlite().expect("Could not open connection to database.");
    sql::initialize(&connection);
    return true;
}


pub fn dupdb_reset_database_from_existing_files(path: PathBuf, duplicate_database: &mut DuplicateDatabase) {
    println!("Reseting database according to files within {:?}", path);
    sql::reset_all_data(&duplicate_database.conn);

    let entries = RecursiveDirIterator::new(&path).expect("Could not load path to reindex database");
    let paths = entries
        .filter(|dir_entry| dir_entry.path().extension().is_some()) // Remove directories, keep files only.
        .map(|file| file.path())
        .collect();
    dupdb_update_hashes_for(paths, duplicate_database);
}

pub fn dupdb_database_path() -> PathBuf {
    if !DEBUGGING_LOCAL {
        Path::new(env!("HOME")).join(NAME_OF_HIDDEN_FOLDER).join(sql::DATABASE_FILE)
    } else {
        Path::new(".").join(NAME_OF_HIDDEN_FOLDER).join(sql::DATABASE_FILE)
    }
}

pub fn dupdb_save_to_file(_duplicate_database: &DuplicateDatabase) {

}

pub fn dupdb_database_load_to_memory() -> DuplicateDatabase {
    let connection = sql::connect_to_sqlite().expect("Unable to connect to sqlite database");
    sql::initialize(&connection);
    DuplicateDatabase {
        conn: connection
    }
}

pub fn dupdb_watch_forever(watch_folder_path: &Path, duplicate_database: &mut DuplicateDatabase) {
    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx).expect("Failed to configure debouncer");
    debouncer.watch(watch_folder_path, RecursiveMode::Recursive).expect("Failed to begin file watch");
    for result in rx {
        match result {
            Ok(debounced_events) => {
                let paths: Vec<PathBuf> = debounced_events.into_iter().filter_map(|event| {
                    match event.kind {
                        EventKind::Remove(_) => Some(event.paths.clone()),
                        EventKind::Create(_) => Some(event.paths.clone()),
                        EventKind::Modify(_) => Some(event.paths.clone()),
                        EventKind::Any => Some(event.paths.clone()),
                        EventKind::Access(_) => None,
                        EventKind::Other => None,
                    }
                }).flatten().collect();
                dupdb_update_hashes_for(paths, duplicate_database);
            },
            Err(error) => eprintln!("Watch error: {:?}", error),
        }
    }
}

pub fn dupdb_update_hashes_for(paths: Vec<PathBuf>, duplicate_database: &mut DuplicateDatabase) {
    let mut duplicates_in_aggregate = Vec::new();
    let mut db_dirty = false;
    for path in paths.iter() {
        let absolute_path = path::absolute(path)
            .expect("Unable to get absolute path for file to hash").to_str()
            .expect("Unexpected file name containining non utf 8 characters found").to_string();
        if !path.exists() {
            duplicate_database.remove(absolute_path);
            db_dirty = true;
        } else {
            // We don't care about directories, only files we can hash. 
            if path.is_dir() {
                continue;
            }
            match fs::read(path) {
                Ok(bytes) => {
                    let hash = seahash::hash(&bytes);
                    duplicate_database.add(hash, absolute_path.clone());
                    if duplicate_database.contains_duplicate_for_hash(hash) {
                        // send notification
                        println!("Duplicate detected {:?} {:?}", absolute_path, hash);
                        duplicates_in_aggregate.push(path.clone());
                        duplicate_database.debug_key(absolute_path.clone());
                        db_dirty = true;
                    }

                },
                Err(error) => {
                    eprintln!("Unexpected failure to read path: {:?} {:?}", error, path);
                }
            }
        }
    };

    if db_dirty {
        if !duplicates_in_aggregate.is_empty() {
            dupdb_notifications_send(duplicates_in_aggregate);
        }
        dupdb_save_to_file(duplicate_database);
    }
}

pub fn dupdb_notifications_send(duplicate_paths: Vec<PathBuf>) {
    if duplicate_paths.is_empty() {
        return;
    }

    let first_image = path::absolute(duplicate_paths[0].clone())
            .expect("Unable to get absolute path for file to hash").to_str()
            .expect("Unexpected file name containining non utf 8 characters found").to_string();

    let mut listing = String::new();
    for dup in duplicate_paths.iter() {
        if let Some(name) = dup.file_name() {
            listing.push_str("\n • ");
            listing.push_str(&name.to_string_lossy());
        };
    }
    
    let handle = Notification::new().summary("Duplicate Files detected")
        .appname(APPNAME)
        .body(&format!("Duplicate files were saved to the watched directory by dupdb.{listing}").to_string())
        .image_path(&first_image) // Shouldn't happen becuase we already grabbed the abs before
        .finalize()
        .show();

        // Error { kind: Msg("Error { code: HRESULT(0x803E0115), message: \"The size of the notification content is too large.\" }") }

    match handle {
        Ok(_) => {},
        Err(kind) => {
            println!("Could not send notification for duplicates {:?}", kind);
        }
    }
}

pub fn dupdb_debug_file_path_print(path: String, duplicate_database: &DuplicateDatabase) {
    let absolute_path = path::absolute(path)
        .expect("Unable to get absolute path for file to hash").to_str()
        .expect("Unexpected file name containining non utf 8 characters found").to_string();
    duplicate_database.debug_key(absolute_path);
}


#[cfg(test)]
mod test {
    
}