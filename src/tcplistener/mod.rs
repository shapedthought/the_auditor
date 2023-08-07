use anyhow::Result;
use regex::Regex;
use std::net::TcpListener;
use std::io::{Read, Write};


pub async fn run_tcp_listener(address: String) -> Result<String> {
    println!("address to test: {}", address);
    let re = Regex::new(r":(\d+)/").unwrap();
    let mut port = String::new();
    if let Some(caps) = re.captures(&address) {
        port = caps[1].to_string();
    } else {
        Err(anyhow::anyhow!("Unable to parse port from address"))?;
    }
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(address)?;

    println!("Listening on {}", listener.local_addr()?);

    let mut data = String::new();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 2048];
                stream.read_exact(&mut buffer)?;
                let request = String::from_utf8_lossy(&buffer[..]);

                if request.starts_with("GET") {
                    let re = Regex::new(r"GET /([^ ]+)").unwrap();
                    data = re.captures(&request).unwrap()[1].to_string();
                    let http_response = "HTTP/1.1 200 OK\r\n\r\n You can close this window now.".to_string();
                    stream.write_all(http_response.as_bytes()).expect("Failed to write to stream");

                    break;
                }
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
    Ok(data)
}