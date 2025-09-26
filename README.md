# Semantic Layer Builder

A Terminal User Interface (TUI) tool built in Rust using [ratatui](https://github.com/ratatui-org/ratatui) for creating semantic layers to deobfuscate TCP protocol messages. This tool provides an interactive interface for analyzing and creating human-readable mappings of obfuscated network communication data.

## Overview

The Semantic Layer Builder is designed to work with TCP message data captured and stored by a companion Python tool. It enables reverse engineers and network analysts to:

- Browse and analyze captured TCP protocol messages
- Create deobfuscation mappings for protocol fields
- Build semantic layers that translate obfuscated identifiers into meaningful names
- Export semantic mappings for reuse and documentation

**Educational Purpose**: This tool is developed for educational purposes to understand network protocols, reverse engineering techniques, and binary communication analysis.

## Architecture

```
┌──────────────────┐    ┌──────────────────────┐    ┌─────────────────────┐
│ GameTCPSniffer   │───▶│   SQLite Database    │───▶│ Semantic Layer      │
│ (Python/TUI)     │    │                      │    │ Builder (Rust/TUI)  │
│                  │    │ • MessagePack Data   │    │                     │
│ • TCP Capture    │    │ • Session Tracking   │    │ • Interactive UI    │
│ • Filtering      │    │                      │    │ • Deobfuscation     │
│ • Storage        │    │                      │    │ • TOML Export       │
└──────────────────┘    └──────────────────────┘    └─────────────────────┘
```

## Features

### Data Management

- **SQLite Integration**: Reads TCP message data from SQLite database
- **MessagePack Decoding**: Handles MessagePack-encoded protocol data
- **Session Tracking**: Groups related messages by session
- **Non-destructive Operations**: Original data remains unchanged

### Interactive Interface

- **Message Browser**: Navigate through captured TCP messages
- **Filtering System**: Filter messages by various criteria (in development)
- **Real-time Preview**: Toggle between obfuscated and deobfuscated views
- **Intuitive TUI**: Clean, responsive terminal interface

### Deobfuscation Engine

- **Key-Value Mapping**: Create custom deobfuscation dictionaries
- **Semantic Layer Building**: Transform technical identifiers into meaningful names
- **TOML Export**: Save semantic mappings for reuse

## Database Schema

The tool expects a SQLite database with the following schema:

```sql
CREATE TABLE IF NOT EXISTS `tcp_proto_messages` (
  `client_ip` TEXT NOT NULL,
  `server_ip` TEXT NOT NULL,
  `proto` TEXT NOT NULL,
  `size` INT NOT NULL,
  `nb_packet` INT NOT NULL,
  `data` TEXT NOT NULL,        -- MessagePack encoded protocol data
  `version` TEXT NOT NULL,
  `hash` TEXT NOT NULL,
  `session` INTEGER,
  timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Installation

### Prerequisites

- Rust 1.56+ with Cargo
- SQLite database populated by the companion Python tool

### From Source

```bash
git clone https://github.com/OpenProtos/semantic-layer-builder
cd semantic-layer-builder
cargo build --release
```

### Usage

```bash
./target/release/semantic-layer-builder path/to/database.db path/to/layer.toml
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Technical Details

### MessagePack Integration

The tool handles MessagePack-encoded protocol data, providing advantages over JSON:

- More compact binary representation
- Better support for binary data types
- Faster serialization/deserialization
- Preservation of data type information

### Performance Considerations

- Lazy loading of large message sets
- Efficient filtering algorithms
- Minimal memory footprint for large databases
- Responsive UI updates during processing

### Security and Ethics

This tool is designed for educational purposes and legitimate security research. Users should:

- Only analyze network traffic they own or have explicit permission to analyze
- Respect applicable laws and regulations
- Use findings responsibly for defensive security purposes

## Limitations

- Currently supports MessagePack-encoded data only
- Filtering system is under active development
- Requires companion Python tool for initial data capture

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) for the excellent TUI framework
- [SQLite](https://sqlite.org/) for reliable data storage
- [MessagePack](https://msgpack.org/) for efficient serialization
- The Rust community for amazing crates and support

## Disclaimer

This tool is intended for educational and legitimate security research purposes only. Users are responsible for ensuring their use complies with applicable laws and regulations. The authors assume no responsibility for misuse of this software.
