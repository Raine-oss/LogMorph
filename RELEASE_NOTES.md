# LogMorph v1.0.0 - Major Release

A high-performance streaming log analyzer and stack trace deduplicator for Minecraft servers (Paper, Purpur, Spigot) and Java applications, built with Rust. Designed for low memory usage, real-time live tailing, and readable error summaries.

This release provides both the **Minecraft Server Plugin** (`LogMorph-1.0.0.jar` / `LogMorph.jar`) with embedded multi-architecture native engines (Linux x86_64, Linux AArch64/ARM64, Windows x86_64) and the **Standalone CLI Executables** for Linux and Windows VPS environments.

---

## What is in this Release?

### 1. Paper / Spigot Minecraft Plugin (`LogMorph-1.0.0.jar`)
- Drop-in `.jar` for your server's `plugins/` directory.
- Works in **Pterodactyl**, Multicraft, and shared hosting panels without requiring SSH access.
- Embedded native Rust engine with multi-platform binaries:
  - Linux x86_64 (`natives/linux-x86_64/liblogmorph.so`)
  - Linux AArch64 / ARM64 (`natives/linux-aarch64/liblogmorph.so` for Oracle Cloud, Ampere, and ARM servers)
  - Windows x86_64 (`natives/windows-x86_64/logmorph.dll`)
  - Graceful fallback with clear warning if running on unbundled platform (e.g. macOS).
- **Asynchronous Execution & Safe Lifecycle**: Log processing runs entirely on background workers and cancels gracefully on server stop (`onDisable()`), preventing tick lag or leaks.
- Commands supported:
  - `/logmorph` or `/lm`: Analyzes `logs/latest.log` and sends full colorized diagnostics to your console.
  - `/lm summary`: Prints condensed statistical summaries.
  - `/lm plugin <Name>`: Filters errors caused by a specific plugin.
  - `/lm export`: Exports structured analysis report to `plugins/LogMorph/report.json`.
  - `/lm archive <file.log.gz>`: Analyzes compressed archived logs directly.

### 2. Standalone CLI (`logmorph-linux-x86_64` & `logmorph-windows-x86_64.exe`)
- Self-contained binaries for Linux x86_64 and Windows x86_64.
- Subcommands supported: `analyze`, `summary`, `inspect`, `watch`, and `export`.
- Real-time `watch` mode with live log streaming and immediate stack trace rendering on incoming errors.
- Standard process exit codes: `0` (Success), `1` (File Not Found), `2` (Invalid Input), `3` (Internal Error).
- Reads files directly (`logmorph logs/latest.log`), compressed `.log.gz` files, or via stdin pipe (`tail -f logs/latest.log | logmorph`).
- Memory ceiling control via `--max-signatures <N>` with first-seen retention and leak-free signature dropping.

### 3. Core Engine Capabilities
- **Deterministic Plugin Frame Classification**: Frame taxonomy distinguishing `Framework`, `Plugin`, and `Unknown` frames without guessing.
- **Root Plugin Frame Resolution**: `find_top_plugin_frame` returns strictly non-framework plugin frames or `None`.
- **Accurate Stack Reconstruction**: Exact deduplication algorithm for `... N more` common suffix frames.
- **Smart Dynamic Normalization**: Selectively normalizes timestamps, UUIDs, memory addresses, and player coordinates, while preserving semantic numbers (HTTP status codes, network ports, versions, and SQL error codes).
- **Collision Protection**: 64-bit hash indexed with structural identity keys to ensure distinct errors are never mistakenly merged.
- **Attribution Classification**: Identifies root cause with explicit states: `confirmed`, `detected_from_stack_frame`, `ambiguous`, and `unknown`.

---

## Installation Quick Links

### For Minecraft Server Owners:
1. Download `LogMorph-1.0.0.jar` (or `LogMorph.jar`).
2. Place it in `plugins/` and restart your server.
3. Type `/lm` in your server console.

### For Linux Users:
```bash
curl -L -o logmorph https://github.com/Raine-oss/LogMorph/releases/download/v1.0.0/logmorph-linux-x86_64
chmod +x logmorph
sudo mv logmorph /usr/local/bin/
```

---

## Author & Contact

Maintained by **Raine-oss**.

- GitHub: [@Raine-oss](https://github.com/Raine-oss)
- Email: [rainebriars@gmail.com](mailto:rainebriars@gmail.com)
- Discord: [1478591836305231975](https://discord.com/users/1478591836305231975)
