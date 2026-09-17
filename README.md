# LogMorph

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Paper](https://img.shields.io/badge/Paper-1.20%2B-blue.svg)](https://papermc.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-success.svg)](https://github.com/Raine-oss/LogMorph/actions)
[![Downloads](https://img.shields.io/github/downloads/Raine-oss/LogMorph/total.svg)](https://github.com/Raine-oss/LogMorph/releases)

A streaming log analyzer and stack trace deduplicator for Minecraft servers and Java applications, built with Rust. Designed for low memory usage, fast processing, and readable error summaries.

Available as both a **Minecraft Server Plugin** (for Pterodactyl and shared panels) and a **Standalone CLI Executable** (for VPS, terminal, and desktop).

---

## Project Status

LogMorph is currently in active development.

### Available
- Streaming log parsing with low memory overhead
- Log level classification (INFO, WARN, ERROR, DEBUG, TRACE)
- Stack trace multiline grouping and nested `Caused by` extraction
- Collision-resistant error signatures and deterministic deduplication
- Smart dynamic normalization preserving semantic numbers (HTTP codes, ports, versions)
- Explicit attribution states (`confirmed`, `detected_from_stack_frame`, `ambiguous`, `unknown`)
- Standardized process exit codes (`0`, `1`, `2`, `3`)
- Command-line interface with subcommands (`analyze`, `summary`, `inspect`, `watch`, `export`)
- Structured JSON output support (`--format json` and `export --output <FILE>`)
- Memory ceiling control via `--max-signatures <N>` with first-seen retention
- Minecraft Paper/Spigot companion plugin with asynchronous execution and graceful shutdown

### In Progress
- Expanded platform packaging (Linux ARM64, macOS)
- Standardized multi-GB performance benchmarks across hardware configurations
- Expanded parser rules for proxy and alternative server software (Velocity, BungeeCord, Fabric)

---

## Platform Support Matrix

| Platform | CLI Executable | Plugin Native Library |
| :--- | :--- | :--- |
| **Linux x86_64** | Supported | Supported |
| **Linux ARM64 (aarch64)** | Supported | Supported (Oracle Cloud, Ampere) |
| **Windows x86_64** | Supported | Supported (CLI) |
| **macOS (Apple Silicon)** | Planned | Planned |

---

## Quick Start for Server Owners

Choose the method that matches your server hosting setup:

### Method A: Minecraft Server Plugin (Best for Pterodactyl, Oracle Cloud & Shared Hosting)

No SSH or root terminal access required. Works directly inside your server panel (including ARM64 Ampere instances).

[![Download Plugin Jar](https://img.shields.io/badge/Download_Plugin-LogMorph--1.0.0.jar-2ea44f?style=for-the-badge&logo=java&logoColor=white)](https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/LogMorph-1.0.0.jar)
[![View Release v1.0.0](https://img.shields.io/badge/GitHub-Release_v1.0.0-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/Raine-oss/LogMorph/releases/tag/v1.0.0)
[![GitHub Packages](https://img.shields.io/badge/GitHub_Packages-LogMorph_1.0.0-blue?style=for-the-badge&logo=github&logoColor=white)](https://github.com/users/Raine-oss/packages?repo_name=LogMorph)

1. Download [`LogMorph-1.0.0.jar`](https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/LogMorph-1.0.0.jar) (or [`LogMorph.jar`](https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/LogMorph.jar)).
2. Upload the `.jar` file into your server's `plugins/` directory.
3. Restart or reload your server.
4. Run commands directly in your Pterodactyl console or in-game:

```text
# Run full log analysis
/logmorph
# (or use the shortcut)
/lm

# Analyze past compressed archived logs (.log.gz)
/lm archive 2026-09-16-1.log.gz

# Export structured JSON report to plugins/LogMorph/report.json
/lm export

# View execution summary tables only
/lm summary

# Filter errors caused by a specific plugin
/lm plugin WorldGuard

# Show help and list of all plugin commands
/lm help
```

---

### Method B: Standalone CLI Binary (VPS, Dedicated Server, or Local PC)

[![Download Linux x86_64](https://img.shields.io/badge/Download_Linux_x86__64-logmorph--linux--x86__64-333333?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/logmorph-linux-x86_64)
[![Download Linux ARM64](https://img.shields.io/badge/Download_Linux_ARM64-logmorph--linux--aarch64-D32F2F?style=for-the-badge&logo=arm&logoColor=white)](https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/logmorph-linux-aarch64)
[![Download Windows Binary](https://img.shields.io/badge/Download_Windows-logmorph--windows--x86__64.exe-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/logmorph-windows-x86_64.exe)

#### Linux (x86_64 or ARM64 / Ampere)
```bash
# For x86_64 Linux:
curl -L -o logmorph https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/logmorph-linux-x86_64

# For ARM64 Linux (Oracle Cloud / Ampere / aarch64):
curl -L -o logmorph https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/logmorph-linux-aarch64

# Make it executable
chmod +x logmorph

# (Optional) Move to system path for global access
sudo mv logmorph /usr/local/bin/
```

#### Windows
1. Download [`logmorph-windows-x86_64.exe`](https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/logmorph-windows-x86_64.exe).
2. Place it in your server folder or run it from Command Prompt / PowerShell:
```cmd
logmorph-windows-x86_64.exe analyze logs\latest.log
```

---

## Input Sources

LogMorph supports multiple input modes:

- **File Path**: Direct reading from disk via buffered streams:
  ```bash
  logmorph analyze logs/latest.log
  ```
- **Standard Input (Pipe)**: Processing streams from pipes:
  ```bash
  cat logs/latest.log | logmorph analyze
  ```
- **Compressed Archives (`.log.gz`)**: Native real-time streaming decompression:
  ```bash
  logmorph analyze logs/2026-09-17-1.log.gz
  ```
  Or via pipes:
  ```bash
  zcat logs/2026-09-17-1.log.gz | logmorph analyze
  ```
- **Live Monitoring (Watch Mode)**: Following live log files as new lines are appended:
  ```bash
  logmorph watch logs/latest.log
  ```

Commands such as `analyze`, `summary`, and `export` stream data until End-Of-File (EOF). The `watch` command operates continuously until interrupted (Ctrl+C).

---

## CLI Commands & Subcommands

```text
logmorph [COMMAND] [OPTIONS]

Commands:
  analyze   Analyze a log file and display aggregated errors [default]
  summary   Display execution summary tables only
  inspect   Inspect full stack trace and details for a specific error (alias: show)
  export    Export structured analysis data to a file or stdout
  watch     Monitor a log file in real-time as lines arrive
  help      Print this message or the help of the given subcommand(s)
```

### Examples

```bash
# 1. Full analysis with ANSI formatting
logmorph analyze logs/latest.log

# 2. View statistical summary table only
logmorph summary logs/latest.log

# 3. Inspect full stack trace and cause chain of error signature #1
logmorph inspect logs/latest.log --error 1

# 4. Export structured JSON report to a file
logmorph export logs/latest.log --output report.json

# 5. Cap maximum tracked error signatures in memory
logmorph analyze logs/latest.log --max-signatures 500

# 6. Filter by plugin or log level
logmorph analyze logs/latest.log --plugin WorldGuard --level ERROR
```

---

## Standard Process Exit Codes

For automation and CI/CD pipelines, LogMorph returns standardized exit codes:

| Exit Code | Status | Description |
| :---: | :--- | :--- |
| **0** | `SUCCESS` | Log analysis completed normally |
| **1** | `FILE_NOT_FOUND` | Specified log file does not exist or cannot be opened |
| **2** | `INVALID_INPUT` | Invalid argument, unknown parameter, or out-of-range inspect index |
| **3** | `INTERNAL_ERROR` | Internal stream reading failure or unexpected parser error |

---

## Error Deduplication & Memory Model

When a plugin encounters a repeating error in a game loop or tick event, logs can accumulate thousands of identical stack traces.

### Smart Normalization
LogMorph does not blindly strip all numbers. It selectively normalizes purely dynamic parameters:
- **Normalized to tokens**: UUIDs (`<UUID>`), timestamps (`<TIME>`), hex memory addresses (`<ADDR>`), and 3D player coordinates (`<COORD>`).
- **Preserved intact**: HTTP status codes (404, 500), network ports (25565, 3306), plugin versions (v1.20.4), and SQL error codes (1045) are kept as part of the error identity.

### Collision Protection
LogMorph uses 64-bit hashes as a fast $O(1)$ index key, but maintains full structural identity keys. If two distinct errors produce the same hash, the engine differentiates them by their structural identity, preventing false merges.

### Attribution Classification
Attribution is classified into four explicit states:
- `confirmed`: Explicitly extracted from Minecraft event failure headers or logger names.
- `detected_from_stack_frame`: Inferred from a single candidate plugin namespace in non-framework frames.
- `ambiguous`: Multiple distinct plugin namespaces appear in non-framework frames.
- `unknown`: No identifiable plugin frames or headers found.

### Memory Ceiling (`--max-signatures`)
LogMorph processes log lines via buffered streaming without buffering the entire file in memory. Working memory is primarily proportional to the number of **unique error signatures** tracked.

To prevent unbounded memory consumption on files containing tens of thousands of distinct exceptions, pass the `--max-signatures <N>` flag:
```bash
logmorph analyze logs/latest.log --max-signatures 1000
```
**Retention Behavior**:
- When the signature limit is reached, incoming errors matching already tracked signatures continue to have their occurrence counters and timelines updated.
- New unseen signatures arriving after the limit is reached are dropped, incrementing the `dropped_signatures` counter.
- The order of log entries determines which signatures are tracked once the cap is reached (first-seen retention).

---

## Output Examples

### Standard Terminal Output
```text
=== Execution Summary ===
┌─────────────────────────────┬───────────────────────────┐
│ Total Lines Processed       │                        18 │
│ Total Log Entries           │                         6 │
│ INFO Messages               │                         5 │
│ WARN Messages               │                         0 │
│ ERROR Messages              │                         1 │
│ DEBUG Messages              │                         0 │
├─────────────────────────────┼───────────────────────────┤
│ Total Exceptions Emitted    │                         1 │
│ Unique Error Signatures     │                         1 │
└─────────────────────────────┴───────────────────────────┘

=== Top Offending Plugins ===
┌────────────────────────────────┬────────────────────────┐
│ Plugin Name                    │            Error Count │
├────────────────────────────────┼────────────────────────┤
│ MyCustomPlugin                 │                      1 │
└────────────────────────────────┴────────────────────────┘

=== Aggregated Error Signatures (1) ===

#1 [Occurrences: 1] org.bukkit.event.EventException (Signature: 0x09e4a6440f4c7d70)
  Plugin: MyCustomPlugin on Event PlayerMoveEvent (Confirmed)
  Timestamp: 12:00:05
  ↳ Root Plugin Frame: com.example.myplugin.listeners.MoveListener.onPlayerMove(MoveListener.java:45)
  Stack Trace (Key Frames):
      · org.bukkit.plugin.java.JavaPluginLoader$1.execute(JavaPluginLoader.java:310)
      · io.papermc.paper.plugin.manager.PaperEventManager.callEvent(PaperEventManager.java:54)
    ▶ com.example.myplugin.listeners.MoveListener.onPlayerMove(MoveListener.java:45)
      · net.minecraft.server.MinecraftServer.tickServer(MinecraftServer.java:1100)
    Caused by: java.lang.NullPointerException
      Cannot invoke "org.bukkit.entity.Player.getName()" because "player" is null
      ▶ com.example.myplugin.services.ScoreboardManager.update(ScoreboardManager.java:88)
      ▶ com.example.myplugin.listeners.MoveListener.onPlayerMove(MoveListener.java:43)
```

### JSON Schema (`--format json` or `export`)
```json
{
  "stats": {
    "total_lines": 18,
    "total_log_messages": 6,
    "info_count": 5,
    "warn_count": 0,
    "error_count": 1,
    "debug_count": 0,
    "total_exceptions": 1,
    "unique_signatures": 1,
    "dropped_signatures": 0
  },
  "top_plugins": [
    {
      "plugin": "MyCustomPlugin",
      "count": 1
    }
  ],
  "aggregated_errors": [
    {
      "signature_hash": 712877452276039024,
      "primary_exception": "org.bukkit.event.EventException",
      "exception_message": null,
      "plugin_name": "MyCustomPlugin",
      "event_name": "PlayerMoveEvent",
      "attribution": "confirmed",
      "top_plugin_frame": {
        "class_name": "com.example.myplugin.listeners.MoveListener",
        "method_name": "onPlayerMove",
        "file_name": "MoveListener.java",
        "line_number": 45,
        "is_native": false
      },
      "occurrences": 1,
      "first_seen": "12:00:05",
      "last_seen": "12:00:05"
    }
  ]
}
```

---

## Plugin Safety

The companion Paper/Spigot plugin follows strict server safety practices:
- **Asynchronous Execution**: All log analysis and native JNI operations execute on background workers (`Bukkit.getScheduler().runTaskAsynchronously`) to prevent tick lag or server freezes.
- **Thread Safety**: Command responses return safely to the main thread via scheduler callbacks.
- **Graceful Shutdown**: All scheduled background tasks are cancelled on plugin disable (`onDisable()`), ensuring clean thread termination during server stops or reloads.
- **Exception Isolation**: Native Rust routines are wrapped in panic catchers (`std::panic::catch_unwind`) to prevent JVM crashes.
- **Read-Only**: The plugin only reads log files. It does not modify server configuration, player data, world files, or gameplay behavior.

---

## Benchmarks

Benchmark suites are implemented using `criterion`:
```bash
cargo bench --bench stream_benchmark -- --test
```
Standardized multi-GB benchmarks (measuring throughput, peak resident memory, and deduplication latency across 1 MB to 1 GB log sets) are actively being prepared.

---

## Build from Source

### 1. Build the Rust Core & CLI
```bash
git clone https://github.com/Raine-oss/LogMorph.git
cd LogMorph

# Build release executable and shared library
cargo build --release
```

### 2. Build the Paper/Spigot Plugin
```bash
# Copy native shared library into plugin resources
mkdir -p plugin/src/main/resources/natives/linux-x86_64
cp target/release/liblogmorph.so plugin/src/main/resources/natives/linux-x86_64/

# Package plugin jar with Maven
cd plugin
mvn clean package
```

### Running Tests
```bash
# Rust unit and integration tests
cargo test

# Java JNI bridge tests
cd plugin && mvn test
```

---

## Governance & Contributing

- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [MIT License](LICENSE)

---

## Author & Contact

LogMorph is maintained by **Raine-oss**.

- **Email**: [rainebriars@gmail.com](mailto:rainebriars@gmail.com)
- **Discord**: [Raine on Discord](https://discord.com/users/1478591836305231975) (User ID: `1478591836305231975`)
- **GitHub**: [@Raine-oss](https://github.com/Raine-oss)

---

## License

LogMorph is free and open-source software licensed under the [MIT License](LICENSE).
