use std::{fs, thread};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use hello_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let thread_pool = ThreadPool::new(4);

    match thread_pool {
        Ok(pool) => {
            for stream in listener.incoming().take(2) {
                match stream {
                    Ok(s) => pool.execute(|| {
                        handle_connection(s);
                    }),
                    Err(err) => println!("Connection not established! {}", err),
                }
            }
            println!("Shutting down the server!");
        }
        Err(err) => println!("Error at: {err}!")
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buffer = BufReader::new(&mut stream);
    // let request = buffer
    //     .lines()
    //     .map(|line| line.unwrap())
    //     .take_while(|res| !res.is_empty())
    //     .collect::<Vec<_>>();
    // println!("Request: {request:#?}");
    let request_line = match buffer.lines().next() {
        None => "".to_string(),
        Some(res) => {
            res.unwrap_or_else(|err| String::from(err.to_string()))
        }
    };

    let (status, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents_result = fs::read_to_string(filename);
    let mut contents = match contents_result {
        Ok(s) => { s }
        Err(err) => { format!("Repsponse error {err} !").to_string() }
    };
    let length = contents.len();

    let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}");
    let response_result = stream.write_all(response.as_bytes());
    match response_result {
        Ok(()) => { println!("200 ok") }
        Err(err) => { println!("cannot create response! {}", err) }
    }
}