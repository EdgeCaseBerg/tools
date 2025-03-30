use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::fs::DirBuilder;
use std::io::Error;
use std::io::ErrorKind;

const HIDDEN_FOLDER_NAME: &str = ".dupdb";

fn main() {
    // Initialize .dupdb in folder.
    dupdb_initialize_hidden_folder(false);

    // Read folder to watch from env
    let folder_to_watch = Path::new(".");
    dupdb_watch_forever(folder_to_watch);
}

fn dupdb_initialize_hidden_folder(use_home_dir: bool) {
    let mut builder = DirBuilder::new();
    let path = if use_home_dir {
        Path::new(env!("HOME")).join(HIDDEN_FOLDER_NAME)
    } else {
        Path::new(".").join(HIDDEN_FOLDER_NAME)
    };
    let mut index_file = path.clone();
    index_file.push("index.dat");

    builder.recursive(true).create(path).expect("Could not create .dupdb database.");
    match File::create_new(&index_file) {
        Ok(file) => {
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
    println!("nothing yet {:?}", watch_folder_path);
}