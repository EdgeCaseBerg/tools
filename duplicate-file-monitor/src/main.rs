use std::path::{ Path, PathBuf };
use std::fs::File;
use std::fs::DirBuilder;
use std::io::ErrorKind;
use std::time::Duration;

use notify::{self, RecursiveMode};
use notify_debouncer_mini::{new_debouncer_opt, Config};

use std::sync::mpsc;

const NAME_OF_HIDDEN_FOLDER: &str = ".dupdb";
const NAME_OF_HASH_FILE: &str = "index.dat";

fn main() {
    // Initialize .dupdb in folder.
    dupdb_initialize_hidden_folder(false);

    // TODO: Read folder to watch from env instead of just .
    let folder_to_watch = Path::new(".");
    dupdb_watch_forever(folder_to_watch);
}

fn dupdb_initialize_hidden_folder(use_home_dir: bool) {
    let mut builder = DirBuilder::new();
    let path = if use_home_dir {
        Path::new(env!("HOME")).join(NAME_OF_HIDDEN_FOLDER)
    } else {
        Path::new(".").join(NAME_OF_HIDDEN_FOLDER)
    };
    let mut index_file = path.clone();
    index_file.push(NAME_OF_HASH_FILE);

    builder.recursive(true).create(path).expect("Could not create .dupdb database.");
    match File::create_new(&index_file) {
        Ok(_) => {
            println!("New index file has been created: {:?}", index_file);
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

fn dupdb_watch_forever(watch_folder_path: &Path) {
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
                dupdb_update_hashes_for(paths);
            },
            Err(error) => eprintln!("Watch error: {:?}", error),
        }
    }
}

fn dupdb_update_hashes_for(paths: Vec<PathBuf>) {
    println!("Would update hashes on {:?}", paths);
}




