mod fixedthreadpool;
use fixedthreadpool::FixedThreadPool;
use std::env;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};
use std::sync::{Arc, Mutex};

fn main() {
    let (sqlite_path, host, port, pool_size) = parse_args();
    println!("starting dupe db with parameters {:?} {:?} {:?} {:?}", sqlite_path, host, port, pool_size);
    let _pool = FixedThreadPool::new(pool_size);
    let listener = match TcpListener::bind(format!("{host}:{port}")) {
        Ok(listener) => listener,
        Err(error) => panic!("Could not bind TcpListener {:?}", error),
    };

    let shutdown_flag = Arc::new(Mutex::new(false));
    let reset_pool_flag = Arc::new(Mutex::new(false));
    let mut fixed_thread_pool = FixedThreadPool::new(pool_size);
    for event in listener.incoming() {
        if fixed_thread_pool.needs_reset() {
            fixed_thread_pool = FixedThreadPool::new(pool_size);
        }
        match event {
            Ok(tcp_stream) => {
                let flag = Arc::clone(&shutdown_flag);
                fixed_thread_pool.execute(move || {
                    if handle_connection(tcp_stream) == ProgramSignal::StopProgram {
                        let mut flag = match flag.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => {
                                flag.clear_poison();
                                poisoned.into_inner()
                            },
                        };
                        *flag = true;
                    }
                });
            },
            Err(error) => eprint!("Could not handle event: {:?}", error),
        };
        let shutdown_flag = match shutdown_flag.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                shutdown_flag.clear_poison();
                poisoned.into_inner()
            },
        };
        if *shutdown_flag {
            break;
        }
    }
}


#[derive(Debug, PartialEq)]
enum ProgramSignal {
    StopProgram,
    ContinueOnMyWayWardSon,
}

fn handle_connection(tcp_stream: TcpStream) -> ProgramSignal {
    let buf_reader = BufReader::new(&tcp_stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();    

    let first_line = http_request.iter().next().map_or("Nonsense!", |s| s);
    match parse_http_request_line(first_line) {
        ("GET", "/shutdown") => {
            send_200("Shutting down...", tcp_stream);
            return ProgramSignal::StopProgram
        },
        ("GET", whatever) => send_200(whatever, tcp_stream),
        ("DELETE", _whatever) => todo!("Not implemeted yet"),
        (method, uri) => send_400(&format!("Invalid request {method} {uri}"), tcp_stream),
    }
    ProgramSignal::ContinueOnMyWayWardSon
}

fn send_200(content: &str, mut tcp_stream: TcpStream) {
    let status = 200;
    let status_line = format!("HTTP/1.1 {status} OK");
    let length = content.len();
    let headers = format!("Content-Length: {length}");
    let response = format!("{status_line}\r\n{headers}\r\n\r\n{content}");
    match tcp_stream.write_all(response.as_bytes()) {
        Ok(_) => return,
        Err(error) => eprintln!("Failed to write response to output {:?}", error)
    }
}

fn send_400(content: &str, mut tcp_stream: TcpStream) {
    let status = 400;
    let status_line = format!("HTTP/1.1 {status} Bad Request");
    let length = content.len();
    let headers = format!("Content-Length: {length}");
    let response = format!("{status_line}\r\n{headers}\r\n\r\n{content}");
    match tcp_stream.write_all(response.as_bytes()) {
        Ok(_) => return,
        Err(error) => eprintln!("Failed to write response to output {:?}", error)
    }
}

fn parse_http_request_line(line: &str) -> (&str, &str) {
    let method_and_uri: Vec<&str> =line
        .split(" ") // https://datatracker.ietf.org/doc/html/rfc2616#autoid-38
        .take(2)
        .collect();
    if method_and_uri.len() != 2 {
        return ("???", "???");
    }
    let method = method_and_uri[0];
    let uri = method_and_uri[1];
    return (method, uri);
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
