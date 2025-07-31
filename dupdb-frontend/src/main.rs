mod fixedthreadpool;
use fixedthreadpool::FixedThreadPool;
use std::env;

fn main() {
    let (sqlite_path, host, port, pool_size) = parse_args();
    println!("{:?} {:?} {:?} {:?}", sqlite_path, host, port, pool_size);
    let _pool = FixedThreadPool::new(pool_size);
}

fn parse_args() -> (String, String, u16, usize) {
    let mut sqlite_path = None;
    let mut host = Some(String::from("127.0.0.1"));
    let mut port = Some(6969);
    let mut pool_size = Some(4);
    let mut args = env::args();
    args.next(); // Skip program name.
    while let Some(argument) = args.next() {
        match &argument[..] {
            "-db" => sqlite_path = args.next(),
            "-p" => port = args.next().map(
                |string| string.parse().expect("Could not parse port as integer")
            ),
            "-h" => host = args.next(),
            "-s" => pool_size = args.next().map(
                |string| string.parse().expect("Could not parse pool size as number")
            ),
            unknown => eprintln!("Unknown flag {unknown}"),
        };
    }
    (
        sqlite_path.expect("Please provide the sqlite database path for dupdb via '-db path'").to_string(),
        host.expect("No host was set, provide -h 127.0.0.1 if unsure").to_string(),
        port.expect("No port was set, please provide a number above 1000 to the -p flag"),
        pool_size.expect("Please provide a pool size for how many worker threads will handle requests with -s 4 or similar")
    )
}
