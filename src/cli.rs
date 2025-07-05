use clap::Parser;

#[derive(Parser, Clone)]
pub struct Cli {
    pub peer_ip: String,
    pub peer_port: String,
    pub local_port: String,
    #[arg(long, short)]
    pub username: Option<String>,
    #[arg(long, short)]
    pub password: Option<String>,
}
