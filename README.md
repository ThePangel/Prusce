# 🔌 Prusce
**P**eer-to-**P**eer **Ru**st **S**ecure **C**hat **E**ncryption

A simple, secure, and colorful peer-to-peer chat application built with Rust. It establishes direct TCP connections between users, bypassing central servers, and provides optional end-to-end encryption for secure communication.

## 🚀 Features

- **Direct P2P**: Connect directly to another user's IP and port.
- **End-to-End Encryption**: Secure your messages with AES-256-GCM. Communication is unencrypted by default unless a password is provided.
- **Challenge-Response Authentication**: Securely authenticates users without sending passwords or hashes over the network.
- **Colorful CLI**: A visually appealing command-line interface with unique colors for each user.
- **Auto-Reconnection**: Automatically retries failed connections.
- **Cross-Platform**: Works on Windows, macOS, and Linux.

## 🛠️ Installation

```bash
git clone https://github.com/thepangel/prusce.git
cd prusce
cargo install --path .
```

## 📖 Usage

The application can run in both encrypted and unencrypted modes.

```bash
# Connect to another user without encryption
prusce <peer_ip> <peer_port> <your_local_port>

# Connect with a custom username
prusce <peer_ip> <peer_port> <your_local_port> --username <your_username>

# Connect with end-to-end encryption (recommended)
prusce <peer_ip> <peer_port> <your_local_port> -u <your_username> -p <your_password>

# Example: Connect to 192.168.1.100 on port 8080, listen on port 3345, with encryption
prusce 192.168.1.100 8080 3345 -u myuser -p "a-very-secret-password"
```

### How It Works
1. **Listens** on your specified local port for an incoming connection from your peer.
2. **Connects** to the peer's IP and port.
3. **Authenticates** using a secure challenge-response handshake if a password is provided.
4. **Encrypts** all messages using AES-256-GCM if a password is provided.
5. **Sends messages** by typing and pressing Enter.
6. **Auto-reconnects** if the connection is lost.

## 🔧 Current Status

- [x] Basic TCP connection
- [x] Message sending and receiving
- [x] Auto-reconnection logic
- [x] CLI argument parsing
- [x] End-to-End Message encryption (AES-256-GCM)
- [x] Secure Challenge-Response Authentication
- [ ] NAT traversal (hole punching server)



<div align="center">
  <strong>Built with Rust 🦀 and 💖 by thepangel ^_____^</strong>
</div>
