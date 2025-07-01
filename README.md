# 🔌 Prusce
**P**eer-to-**P**eer **Ru**st **S**ecure **C**hat **E**ncryption

> **⚠️ Work in Progress** - Basic P2P communication working

A simple peer-to-peer communication tool built with Rust. Direct TCP connections between users without central servers. With (soon) end to end encryption and modern and colorfull cli interface

## 🚀 Features

- **Direct P2P**: Connect directly to another user's IP and port
- **Auto-Reconnection**: Automatically retries failed connections
- **CLI Interface**: Simple command-line tool
- **Cross-Platform**: Works on Windows, macOS, and Linux

## 🛠️ Installation

```bash
git clone https://github.com/yourusername/prusce.git
cd prusce
cargo install --path .
```

## 📖 Usage

```bash
# Connect to another user
prusce <peer_ip> <peer_port> <your_local_port>

# Example: Connect to 192.168.1.100 on port 8080, listen on port 3345
prusce 192.168.1.100 8080 3345
```

### How It Works
1. **Listens** on your specified local port for incoming connections
2. **Connects** to the peer's IP and port
3. **Sends messages** by typing and pressing Enter
4. **Auto-reconnects** if connection is lost

## 🔧 Current Status

- [x] Basic TCP connection
- [x] Message sending and receiving
- [x] Auto-reconnection logic
- [x] CLI argument parsing
- [ ] Message encryption
- [ ] NAT traversal (hole punching server)
- [ ] User authentication



<div align="center">
  <strong>Built with Rust 🦀 and 💖 by thepangel ^_____^</strong>
</div> 
