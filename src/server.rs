use crate::crypto::challenge;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use colored::Colorize;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Duration;

pub async fn handle_client(
    mut stream: TcpStream,
    username: String,
    peer_colors: [u8; 3],
    client_colors: [u8; 3],
    last_sent: Arc<AtomicBool>,
    is_encrypted: bool,
    password: String,
) -> std::io::Result<()> {
    if is_encrypted {
        let challenge = challenge();
        match stream.write_all(&challenge).await {
            Err(e) => {
                eprint!("\r{}", e);
            }
            _ => {}
        };
        let mut buffer = [0; 32];

        match stream.read(&mut buffer).await {
            Ok(_) => {
                if buffer
                    != Sha256::digest([password.as_bytes(), challenge.as_slice()].concat())
                        .as_slice()
                {
                    eprint!("Password doesn't match!!\n");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    std::process::exit(1)
                }
            }
            Err(e) => {
                eprint!("Connection lost: {}. Reconnecting...\n", e);
            }
        };
    }

    eprintln!("has connected!");
    print!(
        "{}",
        username.truecolor(client_colors[0], client_colors[1], client_colors[2])
    );
    io::stdout().flush().unwrap();

    let key_hash = Sha256::digest(&password.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_hash);
    let cipher = Aes256Gcm::new(&key);

    loop {
        let mut final_message: Vec<u8> = Vec::new();

        if is_encrypted {
            let mut nonce_buffer = [0u8; 12];
            if stream.read_exact(&mut nonce_buffer).await.is_err() {
                eprintln!("\nPeer disconnected.");
                break;
            }
            let nonce = Nonce::from_slice(&nonce_buffer);

            let mut buffer = [0; 1024];
            let bytes_read = match stream.read(&mut buffer).await {
                Ok(0) => {
                    eprint!("Client disconnected gracefully\n");

                    break;
                }
                Ok(bytes) => bytes,
                Err(e) => {
                    eprint!("Connection lost: {}. Reconnecting...\n", e);

                    break;
                }
            };

            match cipher.decrypt(nonce, buffer[..bytes_read].as_ref()) {
                Ok(decrypted_data) => final_message = decrypted_data,
                Err(e) => {
                    eprintln!("Decryption failed: {}", e);
                }
            }
        } else {
            let mut buffer = [0; 1024];
            let bytes_read = match stream.read(&mut buffer).await {
                Ok(0) => {
                    eprint!("Client disconnected gracefully\n");
                    break;
                }
                Ok(bytes) => bytes,
                Err(e) => {
                    eprint!("Connection lost: {}. Reconnecting...\n", e);

                    break;
                }
            };
            final_message = buffer[..bytes_read].to_vec();
        }
        let received_data = String::from_utf8_lossy(&final_message);
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

pub async fn run_server_task(
    client_port: String,
    handle_username: String,
    peer_colors: [u8; 3],
    client_colors: [u8; 3],
    last_sent: Arc<AtomicBool>,
    is_encrypted: bool,
    password: String,
) {
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", client_port)).await {
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
                    peer_colors,
                    client_colors,
                    last_sent.clone(),
                    is_encrypted,
                    password.clone(),
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
}
