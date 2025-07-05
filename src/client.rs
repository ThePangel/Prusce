use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use colored::Colorize;
use crossterm::{
    ExecutableCommand,
    cursor::{self, position},
    event::{Event, KeyCode, KeyEvent, poll, read},
};
use sha2::{Digest, Sha256};
use std::io::{self, Write, stdout};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{Duration, sleep},
};

pub async fn run_client_task(
    peer_ip: String,
    peer_port: String,
    username: String,
    password_client: String,
    is_encrypted: bool,
    client_colors: [u8; 3],
    last_sent_client: Arc<AtomicBool>,
) {
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
                Ok(mut stream) => {
                    if is_encrypted {
                        let mut buffer = [0; 32];
                        match stream.read(&mut buffer).await {
                            Ok(_) => {
                                let digest = Sha256::digest(
                                    [password_client.as_bytes(), buffer.as_slice()].concat(),
                                );
                                let finished_challenge = digest.as_slice();
                                match stream.write_all(&finished_challenge).await {
                                    Err(e) => {
                                        eprint!("\r{}", e);
                                    }
                                    _ => {}
                                };
                            }
                            Err(e) => {
                                eprint!("Connection lost: {}. Reconnecting...\n", e);
                            }
                        };
                    }
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
                    eprintln!("\nRetrying... ");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        };
        let mut buffer = String::new();
        let key_hash = Sha256::digest(&password_client.as_bytes());
        let key = Key::<Aes256Gcm>::from_slice(&key_hash);
        let cipher = Aes256Gcm::new(&key);

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
                            #[cfg(windows)]
                            {
                                print!("{}", c);
                                io::stdout().flush().unwrap();
                            }
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
                                #[cfg(windows)]
                                {
                                    print!("\n");
                                    io::stdout().flush().unwrap();
                                }
                                println!(
                                    "{}",
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
                                if is_encrypted {
                                    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

                                    match cipher
                                        .encrypt(&nonce, message_to_send.as_bytes().as_ref())
                                    {
                                        Ok(encrypted_data) => {
                                            if stream.write_all(nonce.as_slice()).await.is_err() {
                                                eprint!("\rConnection lost. Reconnecting...");
                                                break;
                                            }

                                            if stream.write_all(&encrypted_data).await.is_err() {
                                                eprint!("\rConnection lost. Reconnecting...");
                                                break;
                                            };
                                        }
                                        Err(e) => {
                                            eprint!("Error encripting: {}", e)
                                        }
                                    }
                                } else {
                                    if stream.write_all(&message_to_send.as_bytes()).await.is_err()
                                    {
                                        eprint!("\rConnection lost. Reconnecting...");
                                        break;
                                    };
                                }

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
}
