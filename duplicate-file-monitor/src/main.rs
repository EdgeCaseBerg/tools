use std::env;
use std::path::{self, Path, PathBuf };
use std::fs::{self, File, DirBuilder};
use std::io::ErrorKind;
use std::io::Write;
use std::time::Duration;
use std::collections::HashMap;

use notify::{self, RecursiveMode};
use notify_debouncer_mini::{new_debouncer_opt, Config};
use std::sync::mpsc;

use serde::{Serialize, Deserialize};
use rmp_serde::{self, Serializer};

use notify_rust::Notification;

use nav_update::RecursiveDirIterator;

const NAME_OF_HIDDEN_FOLDER: &str = ".dupdb";
const NAME_OF_HASH_FILE: &str = "index.dat";
const APPNAME: &str = "Dup DB";
const DEBUGGING_LOCAL: bool = true;


fn main() {
    let folder_name = env::args().nth(1).unwrap_or("./test".to_string());
    let folder_to_watch = Path::new(&folder_name);

    // Initialize .dupdb in folder.
    let needs_reset = dupdb_initialize_hidden_folder();
    // Load database
    let mut database = dupdb_database_load_to_memory();

    if needs_reset {
        dupdb_reset_database_from_existing_files(folder_to_watch.to_path_buf(), &mut database);
        dupdb_save_to_file(&database);
        println!("Initial database saved to {:?}", folder_to_watch);
    }        

    // if 2 arguments are sent, then second is key to look up for debugging
    // because I'm getting a lot of conflicts on files that aren't actually duplicates.
    if let Some(file_path) = env::args().nth(2) {
        dupdb_debug_file_path_print(file_path, &database);
        return;
    }


    dupdb_watch_forever(folder_to_watch, &mut database);
}

#[derive(Serialize, Deserialize, Debug)]
struct DuplicateDatabase {
    hash_to_files: HashMap<u64, Vec<String>>,
    files_to_hash: HashMap<String, u64>
}

impl DuplicateDatabase {
    fn add(&mut self, hash: u64, full_file_path: String) {
        let entry = self.hash_to_files.entry(hash);
        let existing_files = entry.or_default();
        if !existing_files.contains(&full_file_path) {
            existing_files.push(full_file_path.clone());
        }

        self.files_to_hash.entry(full_file_path).insert_entry(hash);
    }

    fn hash_already_exists(&self, hash: u64) -> bool {
        let contains_hash = self.hash_to_files.contains_key(&hash);
        if !contains_hash {
            return false;
        }

        self.hash_to_files.get(&hash).iter().count() >= 1
    }

    fn remove(&mut self, full_file_path: String) {
        match self.files_to_hash.get(&full_file_path) {
            None => {
                eprintln!("Requested to remove path that wasn't tracked {:?}", full_file_path);
                // Could technically do a full search over all values but that shouldn't
                // be neccesary unless we screw up and access the maps directly.
                // This is normal if we haven't built the index yet and a file is removed from where we're watching.
            },
            Some(hash) => {
                let existing_files = self.hash_to_files.entry(*hash).or_default();
                existing_files.retain(|f| *f != full_file_path);
                self.files_to_hash.remove_entry(&full_file_path);
            }
        }
    }

    fn debug_key(&self, full_file_path: String) {
        match self.files_to_hash.get(&full_file_path) {
            None => {
                println!("Path {:?} not in files_to_hash list", full_file_path);
            },
            Some(hash) => {
                println!("Path {:?} is in list with hash {:?}", full_file_path, hash);
                match self.hash_to_files.get(hash) {
                    None => {
                        println!("Hash {:?} does not have a matching list of files", hash);
                    },
                    Some(existing_files) => {
                        for file_mapped_to_hash in existing_files {
                            println!("Value: {:?}", file_mapped_to_hash);
                        }
                    }
                }
            }
        }
    }
}

/// Returns true if new index was created, false otherwise
fn dupdb_initialize_hidden_folder() -> bool {
    let mut builder = DirBuilder::new();
    let path = dupdb_database_path();
    let mut index_file = path.clone();
    index_file.push(NAME_OF_HASH_FILE);

    builder.recursive(true).create(path.clone()).expect("Could not create .dupdb database.");
    match File::create_new(&index_file) {
        Ok(mut file) => {
            println!("New index file has been created: {:?}", index_file);
            let empty = DuplicateDatabase {
                hash_to_files: HashMap::new(),
                files_to_hash: HashMap::new()
            };
            let mut buf = Vec::new();
            empty.serialize(&mut Serializer::new(&mut buf)).expect("Could not serialize empty DuplicateDatabase");
            file.write_all(&buf).expect("Did not write bytes to file");
            true
        },
        Err(error) => {
            if error.kind() == ErrorKind::AlreadyExists {
                // Good, it exists. Do nothing.
                println!("Index file already exists: {:?}", index_file);
            } else {
                panic!("There was a problem creating the index file: {:?}", error);
            }
            false
        }
    }
}

fn dupdb_reset_database_from_existing_files(path: PathBuf, duplicate_database: &mut DuplicateDatabase) {
    println!("Reseting database according to files within {:?}", path);
    let entries = RecursiveDirIterator::new(&path).expect("Could not load path to reindex database");
    let paths = entries
        .filter(|dir_entry| dir_entry.path().extension().is_some()) // Remove directories, keep files only.
        .map(|file| file.path())
        .collect();
    dupdb_update_hashes_for(paths, duplicate_database);
}

fn dupdb_database_path() -> PathBuf {
    if !DEBUGGING_LOCAL {
        Path::new(env!("HOME")).join(NAME_OF_HIDDEN_FOLDER)
    } else {
        Path::new(".").join(NAME_OF_HIDDEN_FOLDER)
    }
}

fn dupdb_save_to_file(duplicate_database: &DuplicateDatabase) {
    let folder = dupdb_database_path();
    let mut index_file = folder.clone();
    index_file.push(NAME_OF_HASH_FILE);

    let mut file = File::options().read(true).write(true).truncate(true).open(index_file).expect("Could not open index file");
    let mut buf = Vec::new();
    duplicate_database.serialize(&mut Serializer::new(&mut buf)).expect("Could not serialize empty DuplicateDatabase");
    file.write_all(&buf).expect("Did not write bytes to file");
}


fn dupdb_database_load_to_memory() -> DuplicateDatabase {
    let folder = dupdb_database_path();
    let mut index_file = folder.clone();
    index_file.push(NAME_OF_HASH_FILE);

    let handle = File::open(index_file).expect("Could not open index file");
    rmp_serde::from_read(handle).expect("Could not deserialize DuplicateDatabase")
}

fn dupdb_watch_forever(watch_folder_path: &Path, duplicate_database: &mut DuplicateDatabase) {
    let (tx, rx) = mpsc::channel();

    let backend_config = notify::Config::default().with_poll_interval(Duration::from_millis(500));
    let debouncer_config = Config::default()
        .with_timeout(Duration::from_millis(500))
        .with_notify_config(backend_config);
    let mut debouncer = new_debouncer_opt::<_, notify::PollWatcher>(debouncer_config, tx).expect("Failed to configure debouncer");

    debouncer.watcher().watch(watch_folder_path, RecursiveMode::Recursive).expect("Failed to begin file watch");
    for result in rx {
        match result {
            Ok(events) => {
                let paths = events.into_iter().map(|event| event.path).collect();
                dupdb_update_hashes_for(paths, duplicate_database);
            },
            Err(error) => eprintln!("Watch error: {:?}", error),
        }
    }
}

fn dupdb_update_hashes_for(paths: Vec<PathBuf>, duplicate_database: &mut DuplicateDatabase) {
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
                    if duplicate_database.hash_already_exists(hash) {
                        // send notification
                        println!("Duplicate detected {:?}", absolute_path);
                        duplicates_in_aggregate.push(path.clone());
                        duplicate_database.debug_key(absolute_path.clone());
                        db_dirty = true;
                    }

                    duplicate_database.add(hash, absolute_path);
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

fn dupdb_notifications_send(duplicate_paths: Vec<PathBuf>) {
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

    handle.expect("Could not send notification for duplicates");
}

fn dupdb_debug_file_path_print(path: String, duplicate_database: &DuplicateDatabase) {
    let absolute_path = path::absolute(path)
        .expect("Unable to get absolute path for file to hash").to_str()
        .expect("Unexpected file name containining non utf 8 characters found").to_string();
    duplicate_database.debug_key(absolute_path);
}


