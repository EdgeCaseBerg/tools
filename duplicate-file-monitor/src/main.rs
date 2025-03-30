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

const NAME_OF_HIDDEN_FOLDER: &str = ".dupdb";
const NAME_OF_HASH_FILE: &str = "index.dat";
const DEBUGGING_LOCAL: bool = true;

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
        self.hash_to_files.contains_key(&hash)
    }

    fn remove(&mut self, full_file_path: String) {
        match self.files_to_hash.get(&full_file_path) {
            None => {
                eprintln!("Requested to remove path that wasn't tracked {:?}", full_file_path);
                // Could technically do a full search over all values but that shouldn't
                // be neccesary unless we screw up and access the maps directly.
            },
            Some(hash) => {
                let existing_files = self.hash_to_files.entry(*hash).or_default();
                existing_files.retain(|f| *f != full_file_path);
                self.files_to_hash.remove_entry(&full_file_path);
            }
        }
    }
}

fn main() {
    // Initialize .dupdb in folder.
    dupdb_initialize_hidden_folder();

    // Load database
    let mut database = dupdb_database_load_to_memory();

    // TODO: Read folder to watch from env instead of just .
    let folder_to_watch = Path::new("./test");
    dupdb_watch_forever(folder_to_watch, &mut database);
}

fn dupdb_initialize_hidden_folder() {
    let mut builder = DirBuilder::new();
    let path = dupdb_database_path();
    let mut index_file = path.clone();
    index_file.push(NAME_OF_HASH_FILE);

    builder.recursive(true).create(path).expect("Could not create .dupdb database.");
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
        },
        Err(error) => {
            if error.kind() == ErrorKind::AlreadyExists {
                // Good, it exists. Do nothing.
                println!("Index file already exists: {:?}", index_file);
            } else {
                panic!("There was a problem creating the index file: {:?}", error);
            }
        }
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

fn dupdb_database_path() -> PathBuf {
    if !DEBUGGING_LOCAL {
        Path::new(env!("HOME")).join(NAME_OF_HIDDEN_FOLDER)
    } else {
        Path::new(".").join(NAME_OF_HIDDEN_FOLDER)
    }
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
                        duplicates_in_aggregate.push(absolute_path.clone());
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

fn dupdb_notifications_send(duplicate_paths: Vec<String>) {
    let handle = Notification::new().summary("Duplicate Files detected")
        .body("Duplicate files were saved to the watched directory by dupdb, what would you like to do?")
        .action("Ignore", "ignore")
        .action("Remove", "remove")
        .image_path(&duplicate_paths[0])
        .show();
    // sadly wait_for_action is only available on xdg


    // handle.wait_for_action(|action| match action {
    //         "__closed" | "ignore" => {
    //             println!("Closed or ignored");
    //             handle.close();
    //         }, 
    //         "remove" => {
    //             println!("Remove please {:?}", duplicate_paths)
    //         },
    //         _ => ()
    //     });
}




