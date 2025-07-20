use std::path::Path;

mod sql;
mod dupdb;

use dupdb::*;

fn main() {
    let folder_name = env::args().nth(1).unwrap_or("./test".to_string());
    let folder_to_watch = Path::new(&folder_name);

    // Initialize .dupdb in folder.
    let needs_reset = dupdb_initialize_hidden_folder();

    // Load database
    let mut database = dupdb_database_load_to_memory();

    if needs_reset {
        dupdb_reset_database_from_existing_files(folder_to_watch.to_path_buf(), &mut database);
        println!("Initial database saved to {:?}", folder_to_watch);
    }        

    // if 2 argumetns are sent, then second is key to look up for debugging
    // because I'm getting a lot of conflicts on files that aren't actually duplicates.    
    if let Some(file_path) = env::args().nth(2) {
        dupdb_debug_file_path_print(file_path, &database);
        return;
    }

    dupdb_watch_forever(folder_to_watch, &mut database);
}

