use notify::{recommended_watcher, Event, RecursiveMode, Result, Watcher};
use std::sync::mpsc;
use std::path::Path;
use std::fs;

fn main() -> Result<()> {
    // TODO get this from the arguments
    let path = Path::new("./data/text");

    let (tx, rx) = mpsc::channel::<Result<Event>>();

    // Use recommended_watcher() to automatically select the best implementation
    // for your platform. The `EventHandler` passed to this constructor can be a
    // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
    // another type the trait is implemented for.
    let mut watcher = notify::recommended_watcher(tx)?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.

    watcher.watch(path, RecursiveMode::Recursive)?;
    // Block forever, printing out events as they come in
    for res in rx {
        match res {
            Ok(event) => {
                println!("event: {:?}", event);
                let s = get_data_from_file(path);
                println!("{:?}", s);
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn get_data_from_file(path: &Path) -> String {
    let s = fs::read_to_string(path).expect("this is a bad idea");
    return s
}