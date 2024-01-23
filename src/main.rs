use std::env::current_dir;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:5500").expect("Port 5500 is already in use");
    println!("Server is available at http://127.0.0.1:5500");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap(); // "GET /path/to/file_or_dir HTTP/1.1\r\n"
    if line.is_empty() {
        return;
    }

    let relative_path = line.split_whitespace().nth(1).unwrap(); // "/path/to/file_or_dir"
    let relative_path = if relative_path.len() == 1 {
        "" // "/" -> ""
    } else {
        relative_path
    };
    let full_path = format!("{}{}", current_dir().unwrap().display(), relative_path);

    let response = match fs::metadata(&full_path).map(|attr| attr.is_dir()) {
        Ok(true) => {
            let contents = fs::read_dir(&full_path)
                .unwrap()
                .fold(String::new(), |acc, entry| {
                    let link = format!(
                        "{}/{}",
                        relative_path,
                        entry.unwrap().file_name().to_str().unwrap()
                    );
                    format!("{acc}<a href={link}>{link}</a><br>")
                });
            format!("HTTP/1.1 200 OK\r\n\r\n{}", contents)
        }
        Ok(false) => {
            let contents = fs::read_to_string(&full_path).unwrap_or("cannot read file".to_string());
            format!("HTTP/1.1 200 OK\r\n\r\n{}", contents)
        }
        Err(err) => {
            println!("Error: {}", err);
            println!("Path: {}", full_path);
            "HTTP/1.1 404 NOT FOUND\r\n\r\nNOT FOUND".to_string()
        }
    };

    write!(stream, "{}", response).unwrap();
    stream.flush().unwrap();
}
