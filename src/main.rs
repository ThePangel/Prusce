use clap::Parser;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};

#[derive(Parser, Clone)]
struct Cli {
    peer_ip: String,

    peer_port: String,

    local_port: String,
}
async fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    loop {
        let mut buffer = [0; 1024];
        let bytes_read = match stream.read(&mut buffer).await {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Connection lost: {}. Reconnecting...", e);
                break;
            }
        };

        if bytes_read == 0 {
            println!("Client disconnected gracefully");
            break;
        }

        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("{}", received_data);
    }
    Ok(())
}
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    let peer_ip = args.peer_ip.clone();
    let client_port = args.local_port.clone();
    let peer_port = args.peer_port.clone();
    drop(args);

    let server_task = tokio::spawn(async move {
        let listener = match TcpListener::bind(format!("127.0.0.1:{}", client_port)).await {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed to bind to port: {}", e);
                return;
            }
        };

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    if let Err(e) = handle_client(stream).await {
                        eprintln!("Error handling client: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    });

    let client_task = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(stdin);
        loop {
            let mut stream = loop {
                match TcpStream::connect(format!("{}:{}", &peer_ip, &peer_port)).await {
                    Ok(stream) => {
                        println!("Connected successfully!");
                        break stream;
                    }
                    Err(e) => {
                        eprint!("Connection failed: {}. \n Retrying...", e);
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
            };

            loop {
                let mut buffer = String::new();
                match reader.read_line(&mut buffer).await {
                    Ok(0) => return,
                    Ok(_) => {
                        if let Err(e) = stream.write_all(buffer.as_bytes()).await {
                            println!("Write error: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Input error: {}", e);
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = server_task => {},
        _ = client_task => {},
    }

    Ok(())
}
