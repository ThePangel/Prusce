use clap::Parser;
use colored::Colorize;
use local_ip_address::local_ip;
use rand::prelude::*;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};

#[derive(Parser, Clone)]
struct Cli {
    peer_ip: String,
    peer_port: String,
    local_port: String,
    #[arg(long, short)]
    username: Option<String>,
}

async fn handle_client(
    mut stream: TcpStream,
    username: String,
    peer_colors: [u8; 3],
    client_colors: [u8; 3],
) -> std::io::Result<()> {
    eprintln!("has connected!");
    print!("{}", username.truecolor(client_colors[0], client_colors[1], client_colors[2]));
    io::stdout().flush().unwrap();
    loop {
        let mut buffer = [0; 1024];
        let bytes_read = match stream.read(&mut buffer).await {
            Ok(bytes) => bytes,
            Err(e) => {
                print!("Connection lost: {}. Reconnecting...\n", e);
                io::stdout().flush().unwrap();
                break;
            }
        };

        if bytes_read == 0 {
            print!("Client disconnected gracefully\n");
            io::stdout().flush().unwrap();
            break;
        }

        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        let printed_data: Vec<&str> = received_data.split_inclusive('$' ).collect();
        print!(
            "\r{}{}",
            printed_data[0].truecolor(peer_colors[0], peer_colors[1], peer_colors[2]),
            printed_data[1]
        );
        print!(
            "{}",
            username.truecolor(client_colors[0], client_colors[1], client_colors[2])
        );

        io::stdout().flush().unwrap();
    }
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = Cli::parse();
    let client_local_ip = local_ip().unwrap();
    if args.username.is_none() {
        args.username = Some(client_local_ip.to_string());
    }

    let username = format!(
        "{}@{}:~$ ",
        args.username.as_ref().unwrap(),
        client_local_ip.to_string()
    );
    let peer_ip = args.peer_ip.clone();
    let client_port = args.local_port.clone();
    let peer_port = args.peer_port.clone();
    let handle_username = username.clone();
    let client_colors: [u8; 3] = [
        255, 
        rand::rng().random_range(0..125),   
        rand::rng().random_range(0..125),   
    ];
    let peer_colors: [u8; 3] = [
        rand::rng().random_range(0..125),   
        rand::rng().random_range(0..125),   
        255, 
    ];
    let client_ser_colors = client_colors.clone();
    drop(args);

    let server_task = tokio::spawn(async move {
        let listener = match TcpListener::bind(format!("127.0.0.1:{}", client_port)).await {
            Ok(listener) => listener,
            Err(e) => {
                if e.to_string() == "invalid port value" {
                    eprintln!("Failed to bind to port: Invalid local port value");
                    return;
                }
                eprintln!("Failed to bind to port: {}", e);
                return;
            }
        };

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    if let Err(e) = handle_client(
                        stream,
                        handle_username.clone(),
                        peer_colors.clone(),
                        client_ser_colors.clone(),
                    )
                    .await
                    {
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
        print!(
            "{}",
            username.truecolor(client_colors[0], client_colors[1], client_colors[2])
        );
        io::stdout().flush().unwrap();

        loop {
            let mut stream = loop {
                match TcpStream::connect(format!("{}:{}", &peer_ip, &peer_port)).await {
                    Ok(stream) => {
                        break stream;
                    }
                    Err(e) => {
                        if e.to_string() == "invalid port value" {
                            eprintln!("Connection failed: Invalid peer port value.");
                            return;
                        } else if e.to_string() == "No such host is known. (os error 11001)" {
                            eprintln!("Connection failed: {}. \nProbably wrong ip address", e);
                            return;
                        }
                        eprintln!("Retrying... ");
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
                        print!(
                            "{}",
                            username.truecolor(
                                client_colors[0],
                                client_colors[1],
                                client_colors[2]
                            )
                        );
                        io::stdout().flush().unwrap();
                        let message_to_send = format!("{}{}", username, buffer);
                        if stream.write_all(message_to_send.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Err(_e) => {
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
