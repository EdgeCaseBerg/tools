mod fixedthreadpool;
use fixedthreadpool::FixedThreadPool;
use std::env;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};
use std::sync::{Arc, Mutex};
use std::fs;
use form_urlencoded::parse;
use urlencoding::decode;

use duplicate_file_monitor::rusqlite::{Connection, OpenFlags};


fn main() {
    let (sqlite_path, host, port, pool_size) = parse_args();
    println!("starting dupe db with parameters {:?} {:?} {:?} {:?}", sqlite_path, host, port, pool_size);

    // Verify connection first (this can panic) so that we don't 
    // have to worry about unbinding the TCP port in a moment
    let db_connection = open_db_connection(&sqlite_path);
    drop(db_connection);

    // this can panic
    let mut fixed_thread_pool = FixedThreadPool::new(pool_size);

    // HTTP setup
    let listener = match TcpListener::bind(format!("{host}:{port}")) {
        Ok(listener) => listener,
        Err(error) => panic!("Could not bind TcpListener {:?}", error),
    };


    // Execution pool and the "job" that runs per request.
    let shutdown_flag = Arc::new(Mutex::new(false));
    for event in listener.incoming() {
        if fixed_thread_pool.needs_reset() {
            fixed_thread_pool = FixedThreadPool::new(pool_size);
        }
        match event {
            Ok(tcp_stream) => {
                let flag = Arc::clone(&shutdown_flag);
                let sqlite_path = sqlite_path.clone();

                fixed_thread_pool.execute(move || {
                    let db_connection = open_db_connection(&sqlite_path);
                    if handle_connection(tcp_stream, db_connection) == ProgramSignal::StopProgram {
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

/// Panics if db cant be opened.
fn open_db_connection(sqlite_path: &str) -> Connection {
    match Connection::open_with_flags(
        sqlite_path, 
        OpenFlags::SQLITE_OPEN_READ_ONLY |
        OpenFlags::SQLITE_OPEN_NO_MUTEX  |
        OpenFlags::SQLITE_OPEN_URI
    ) {
        Err(error) => {
            panic!("Cannot open database connection {error}");
        },
        Ok(conn) => conn
    }
}


#[derive(Debug, PartialEq)]
enum ProgramSignal {
    StopProgram,
    ContinueOnMyWayWardSon,
}

fn get_http_from(tcp_stream: &TcpStream) -> (Vec<String>, Option<Vec<u8>>) {
    let mut buf_reader = BufReader::new(tcp_stream);
    let mut headers: Vec<String> = vec![];
    loop {
        let mut utf8_line = String::new();
        match buf_reader.read_line(&mut utf8_line) {
            Ok(0) => {
                return (headers, None);
            },
            Ok(_) => {
                if utf8_line.is_empty() || utf8_line == "\r\n" {
                    break;
                }
                headers.push(utf8_line);
            },
            Err(error) => {
                // I suppose we have received non utf-8 in the header lines.
                // Which is QUITE odd... let us abort with whatever we have.
                eprintln!("Error while reading HTTP request: {error}");
                return (headers, None);
            }
        }
    }

    let content_length_line = match headers.iter().find(|line| line.starts_with("Content-Length")) {
        None => return (headers, None),
        Some(line) => {
            let raw_content_length = line.split(" ").skip(1).take(1).collect::<String>();
            match raw_content_length.trim().parse() {
                Err(bad_number_error) => {
                    eprintln!("Can't parse content length header. {bad_number_error}");
                    return (headers, None);
                },
                Ok(good) => good
            }
        }
    };

    let body_bytes: Vec<u8> = buf_reader.bytes().take(content_length_line).map(|u8| u8.unwrap()).collect();
    (headers, Some(body_bytes))
}

fn handle_connection(tcp_stream: TcpStream, readonly_connection: Connection) -> ProgramSignal {
    let (http_request_headers, maybe_http_body) = get_http_from(&tcp_stream);

    let first_line = http_request_headers.iter().next().map_or("Nonsense!", |s| s);
    match parse_http_request_line(first_line) {
        ("GET", "/duplicates") => {
            let duplicate_tuples = get_dups(&readonly_connection);
            let mut response_body = String::new();
            for (hash,file_path) in duplicate_tuples {
                response_body.push_str(&format!("{hash}\n{file_path}\n\n"));
            }
            send_200(&response_body, tcp_stream);
        }
        ("POST", "/remove") => {
            if maybe_http_body.is_none() {
                send_400("Invalid request, send an http body fool!", tcp_stream);
                return ProgramSignal::ContinueOnMyWayWardSon;
            }
            let http_body = maybe_http_body.unwrap();
            let path_tuple_to_remove = parse(&http_body).into_owned().find(|(name, _)| {
                name == "path"
            });
            if path_tuple_to_remove.is_none() {
                send_400("Invalid request, no path found in form body", tcp_stream);
                return ProgramSignal::ContinueOnMyWayWardSon;
            }

            let (_, path_to_remove) = path_tuple_to_remove.unwrap();
            if !fs::exists(&path_to_remove).unwrap_or(false) {
                send_400(&format!("No file exists at path {path_to_remove}"), tcp_stream);
                return ProgramSignal::ContinueOnMyWayWardSon;
            }

            let _ = fs::remove_file(&path_to_remove);
            send_303_home(tcp_stream);
        }
        ("GET", "/shutdown") => {
            send_200("Shutting down...", tcp_stream);
            return ProgramSignal::StopProgram
        },
        ("GET", "/") => {
            match fs::read_to_string("index.html") {
                Ok(bytes) => send_200(&bytes, tcp_stream),
                Err(error) => send_400(&format!("{error}"), tcp_stream),
            };
        },
        ("GET", whatever) => {
            let file_reference = &whatever[1..];
            let file_reference = match decode(file_reference) {
                Ok(cow) => cow.into_owned(),
                Err(error) => {
                    eprintln!("Can't read file reference properly: {error}");
                    String::new()
                }
            };
            if !fs::exists(&file_reference).unwrap_or(false) {
                send_400("No file there m8", tcp_stream);
            } else {
                if file_reference.ends_with(".html") {
                    match fs::read_to_string(file_reference) {
                        Ok(content) => {
                            send_200(&content, tcp_stream)
                        },
                        Err(error) => send_400(&format!("{error}"), tcp_stream),
                    };    
                } else {
                    match fs::read(file_reference) {
                        Ok(bytes) => {
                            send_200_bytes(&bytes, tcp_stream)
                        },
                        Err(error) => send_400(&format!("{error}"), tcp_stream),
                    };
                }
            }
        }
        (method, uri) => send_400(&format!("Invalid request {method} {uri}"), tcp_stream),
    }
    ProgramSignal::ContinueOnMyWayWardSon
}

const SELECT_ALL_DUPES: &str = "
SELECT hash, file_path
FROM dupdb_filehashes
WHERE hash IN (
    SELECT hash
    FROM dupdb_filehashes
    GROUP BY hash
    HAVING COUNT(DISTINCT file_path) > 1
)
ORDER BY hash
";

fn get_dups(conn: &Connection) -> Vec<(String, String)> {
    let mut statement = conn.prepare_cached(SELECT_ALL_DUPES)
        .expect("Could not fetch prepared select_dups query");

    let rows = statement.query_map([], |row| {
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

fn send_200_bytes(content: &Vec<u8>, mut tcp_stream: TcpStream) {
    let status = 200;
    let status_line = format!("HTTP/1.1 {status} OK");
    let length = content.len();
    let headers = format!("Content-Length: {length}\r\nContent-type: application/octet-stream");
    let response = format!("{status_line}\r\n{headers}\r\n\r\n");
    match tcp_stream.write_all(response.as_bytes()) {
        Ok(_) => {
            let _ = tcp_stream.write_all(content);
        },
        Err(error) => {
            eprintln!("Failed to write response to output {:?}", error);
        }
    }
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

fn send_303_home(mut tcp_stream: TcpStream) {
    let status = 303;
    let status_line = format!("HTTP/1.1 {status} SEE OTHER");
    let headers = format!("Location: /");
    let response = format!("{status_line}\r\n{headers}\r\n\r\n");
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
