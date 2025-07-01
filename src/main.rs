use clap::Parser;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[derive(Parser, Clone)]
struct Cli {
    ip: String,

    port: String,
}
fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    println!("Server received: {:?}", &buffer[..bytes_read]);

    // Echo back the data
    stream.write(&buffer[..bytes_read])?;
    Ok(())
}
fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    //let server_port = args.port.clone();
    let client_ip = args.ip.clone();
    let client_port = args.port.clone();

    drop(args);
    thread::spawn(move || {
        let listener = match TcpListener::bind(format!("127.0.0.1:3345")) {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed to bind to port: {}", e);
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    });
    thread::spawn(move || {
        loop {
            let mut stream = match TcpStream::connect(format!("{}:{}", &client_ip, &client_port)) {
                Ok(stream) => stream,
                Err(e) => {
                    eprint!("Error: {}", e);
                    continue; 
                }
            };
            let _ = stream.write(&[1]);
            
        }
        
    });
    loop{thread::sleep(Duration::from_millis(1))};
   
}
