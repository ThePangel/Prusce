mod cli;
mod client;
mod crypto;
mod server;

use clap::Parser;
use crossterm::style::Stylize;
use local_ip_address::local_ip;
use rand::{Rng, rng};
use std::sync::{Arc, atomic::AtomicBool};

use cli::Cli;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = Cli::parse();
    let client_local_ip = local_ip().unwrap_or("127.0.0.1".parse().unwrap());
    if args.username.is_none() {
        args.username = Some(client_local_ip.to_string());
    }
    let password: String = if args.password.is_none() {
        eprint!(
            "{}\n",
            "WARNING ⚠️!! No password provided, sending messages without encryption!"
                .on_dark_red()
                .yellow()
        );
        String::new()
    } else {
        format!("{}", args.password.clone().unwrap())
    };

    let is_encrypted = !password.is_empty();
    let password_client = password.clone();
    let username = format!(
        "{}@{}:~$ ",
        args.username.as_ref().unwrap(),
        client_local_ip.to_string()
    );
    let peer_ip = args.peer_ip.clone();
    let client_port = args.local_port.clone();
    let peer_port = args.peer_port.clone();
    let handle_username = username.clone();
    let mut rng = rng();
    let client_colors: [u8; 3] = [
        rng.random_range(0..255),
        rng.random_range(0..255),
        rng.random_range(0..255),
    ];
    let peer_colors: [u8; 3] = [
        rng.random_range(0..255),
        rng.random_range(0..255),
        rng.random_range(0..255),
    ];
    let client_ser_colors = client_colors.clone();

    let last_sent = Arc::new(AtomicBool::new(false));
    let last_sent_server = last_sent.clone();
    let last_sent_client = last_sent.clone();

    drop(args);

    let server_task = tokio::spawn(server::run_server_task(
        client_port,
        handle_username,
        peer_colors,
        client_ser_colors,
        last_sent_server,
        is_encrypted,
        password,
    ));

    let client_task = tokio::spawn(client::run_client_task(
        peer_ip,
        peer_port,
        username,
        password_client,
        is_encrypted,
        client_colors,
        last_sent_client,
    ));

    tokio::select! {
        _ = server_task => {},
        _ = client_task => {},
    }

    Ok(())
}
