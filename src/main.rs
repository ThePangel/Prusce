use clap::Parser;
use colored::Colorize;
use crossterm::cursor::position;
use crossterm::event::{Event, KeyCode, KeyEvent, poll, read};
use crossterm::{ExecutableCommand, cursor};
use local_ip_address::local_ip;
use rand::prelude::*;
use std::io::{self, Write, stdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    last_sent: Arc<AtomicBool>,
) -> std::io::Result<()> {
    eprintln!("has connected!");
    print!(
        "{}",
        username.truecolor(client_colors[0], client_colors[1], client_colors[2])
    );
    io::stdout().flush().unwrap();
    loop {
        let mut buffer = [0; 1024];
        let bytes_read = match stream.read(&mut buffer).await {
            Ok(bytes) => bytes,
            Err(e) => {
                eprint!("Connection lost: {}. Reconnecting...\n", e);
               
                break;
            }
        };

        if bytes_read == 0 {
            eprint!("Client disconnected gracefully\n");
            
            break;
        }

        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);

        let printed_data: Vec<&str> = received_data.split_inclusive('$').collect();
        print!(
            "\r\x1b[2K{}{}",
            printed_data[0].truecolor(peer_colors[0], peer_colors[1], peer_colors[2]),
            printed_data[1]
        );
        print!(
            "\n{}",
            username.truecolor(client_colors[0], client_colors[1], client_colors[2])
        );

        io::stdout().flush().unwrap();
        last_sent.store(true, Ordering::Relaxed);
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

    let last_sent = Arc::new(AtomicBool::new(false));
    let last_sent_server = last_sent.clone();
    let last_sent_client = last_sent.clone();

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
                    match handle_client(
                        stream,
                        handle_username.clone(),
                        peer_colors.clone(),
                        client_ser_colors.clone(),
                        last_sent_server.clone(),
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error handling client: {}", e);
                            
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                    io::stdout().flush().unwrap();
                }
            }
        }
    });

    let client_task = tokio::spawn(async move {
        let mut stdout = stdout();

        let _ = stdout.execute(cursor::SetCursorStyle::BlinkingBlock);
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
                        } else {
                            eprint!("{}", e)
                        }
                        eprintln!("Retrying... ");
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
            };
            let mut buffer = String::new();

            loop {
                if last_sent_client.load(Ordering::Relaxed) {
                    print!("{}", buffer);
                    io::stdout().flush().unwrap();
                    last_sent_client.store(false, Ordering::Relaxed);
                };
                if poll(Duration::from_millis(0)).unwrap_or(false) {
                    match read().unwrap() {
                        Event::Key(KeyEvent {
                            code,
                            kind: crossterm::event::KeyEventKind::Press,
                            ..
                        }) => match code {
                            KeyCode::Char(c) => {
                                buffer.push(c);
                                print!("{}", c);
                                io::stdout().flush().unwrap();
                            }
                            KeyCode::Backspace => match position() {
                                Ok(value) => {
                                    if value.0 >= (username.len() + 1) as u16 {
                                        buffer.pop();
                                        print!("\x08 \x08");
                                        io::stdout().flush().unwrap();
                                    }
                                }
                                Err(e) => {
                                    eprint!("{}", e);
                                }
                            },
                            KeyCode::Enter => {
                                if !buffer.is_empty() {
                                    println!(
                                        "\n{}",
                                        username.truecolor(
                                            client_colors[0],
                                            client_colors[1],
                                            client_colors[2]
                                        )
                                    );

                                    io::stdout().flush().unwrap();
                                    stdout.execute(cursor::MoveUp(1)).unwrap();
                                    stdout
                                        .execute(cursor::MoveRight(username.len() as u16))
                                        .unwrap();
                                    let message_to_send = format!("{}{}", username, buffer);

                                    match stream.write_all(message_to_send.as_bytes()).await {
                                        Err(e) => {
                                            eprint!("\r{}", e);
                                        }
                                        _ => {}
                                    };

                                    buffer = String::new();
                                }
                            }

                            KeyCode::Esc => {
                                return;
                            }
                            _ => {}
                        },

                        _ => {}
                    }
                } else {
                    tokio::task::yield_now().await;
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
