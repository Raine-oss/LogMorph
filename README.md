# LogMorph

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-success.svg)](#)
[![Downloads](https://img.shields.io/github/downloads/Raine-oss/LogMorph/total.svg)](#)

A zero-overhead, streaming log analyzer and stack trace deduplicator built specifically for Minecraft servers (Paper, Purpur, Spigot) and Java applications.

---

## Quick Start for Server Owners

If your server crashed or your console is spamming errors, you can run LogMorph immediately to find the root cause.

### 1. Download & Install

#### Linux (VPS / Dedicated Server)
```bash
# Download the latest binary directly
curl -L -o logmorph https://github.com/Raine-oss/LogMorph/releases/latest/download/logmorph

# Make it executable
chmod +x logmorph

# (Optional) Move to your system path for global access
sudo mv logmorph /usr/local/bin/
```

#### Windows
1. Download `logmorph.exe` from the [Latest Release](https://github.com/Raine-oss/LogMorph/releases/latest).
2. Place it in your server folder or run it from Command Prompt / PowerShell.

---

### 2. Basic Usage

Run LogMorph directly against your server's log file:

```bash
# Analyze your latest server log
logmorph logs/latest.log

# Monitor live server logs in real-time
tail -f logs/latest.log | logmorph

# Read an archived, compressed log
zcat logs/2026-09-17-1.log.gz | logmorph
```

#### Handy Options
```bash
# Filter by a specific plugin name
logmorph logs/latest.log --plugin WorldGuard

# Show high-level summary tables only
logmorph logs/latest.log --summary-only

# Filter by minimum log level
logmorph logs/latest.log --level ERROR

# Disable colors for plain text files or script outputs
logmorph logs/latest.log --no-color
```

---

## What Problem Does LogMorph Solve?

When a plugin throws an error in an event loop or a tick task, your console often gets flooded with thousands of identical lines. Reading through a 500 MB log file manually to find what went wrong is slow, difficult, and can freeze standard text editors.

LogMorph fixes this by:

1. **Deduplicating Repeated Errors**: If an exception appears 5,000 times, LogMorph compresses it into a single clean entry showing the exact occurrence count, the first time it happened, and the last time it happened.
2. **Filtering Framework Noise**: Minecraft logs contain dozens of internal frames (`net.minecraft`, `org.bukkit`, `com.destroystokyo.paper`, `java.lang.reflect`). LogMorph identifies and highlights the exact line in your plugin that failed, while muting the framework boilerplate.
3. **Streaming with Constant Memory**: LogMorph processes logs line-by-line via buffered streams. It uses the exact same tiny memory footprint whether analyzing a 50 KB file or a 10 GB file.

---

## Output Example

```text
=== Execution Summary ===
+-----------------------------+---------------------------+
| Total Lines Processed       |                        18 |
| Total Log Entries           |                         6 |
| INFO Messages               |                         5 |
| WARN Messages               |                         0 |
| ERROR Messages              |                         1 |
| DEBUG Messages              |                         0 |
+-----------------------------+---------------------------+
| Total Exceptions Emitted    |                         1 |
| Unique Error Signatures     |                         1 |
+-----------------------------+---------------------------+

=== Top Offending Plugins ===
+--------------------------------+------------------------+
| Plugin Name                    |            Error Count |
+--------------------------------+------------------------+
| MyCustomPlugin                 |                      1 |
+--------------------------------+------------------------+

=== Aggregated Error Signatures (1) ===

#1 [Occurrences: 1] org.bukkit.event.EventException (Signature: 0x09e4a6440f4c7d70)
  Plugin: MyCustomPlugin on Event PlayerMoveEvent
  Message: null
  Timestamp: 12:00:05
  Root Plugin Frame: com.example.myplugin.listeners.MoveListener.onPlayerMove(MoveListener.java:45)
  Stack Trace (Key Frames):
      . org.bukkit.plugin.java.JavaPluginLoader$1.execute(JavaPluginLoader.java:310)
      . io.papermc.paper.plugin.manager.PaperEventManager.callEvent(PaperEventManager.java:54)
    > com.example.myplugin.listeners.MoveListener.onPlayerMove(MoveListener.java:45)
      . net.minecraft.server.MinecraftServer.tickServer(MinecraftServer.java:1100)
    Caused by: java.lang.NullPointerException
      Cannot invoke "org.bukkit.entity.Player.getName()" because "player" is null
      > com.example.myplugin.services.ScoreboardManager.update(ScoreboardManager.java:88)
      > com.example.myplugin.listeners.MoveListener.onPlayerMove(MoveListener.java:43)
```

---

## Build from Source

If you prefer building from source, you only need Rust and Cargo installed:

```bash
# Clone the repository
git clone https://github.com/Raine-oss/LogMorph.git
cd LogMorph

# Build the release binary
cargo build --release

# The compiled binary will be located at:
# target/release/logmorph
```

To run the built-in test suite:
```bash
cargo test
```

To run the benchmark suite:
```bash
cargo bench --bench stream_benchmark -- --test
```

---

## Author & Contact

LogMorph is maintained by **Raine-oss**.

- **Email**: [rainebriars@gmail.com](mailto:rainebriars@gmail.com)
- **Discord**: [Raine on Discord](https://discord.com/users/1478591836305231975) (User ID: `1478591836305231975`)
- **GitHub**: [@Raine-oss](https://github.com/Raine-oss)

---

## License

LogMorph is 100% free and open-source software released under the [MIT License](LICENSE). You are welcome to use, modify, and distribute it freely.
