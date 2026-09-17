# LogMorph v0.1.0 - Initial Release

A streaming log analyzer and stack trace deduplicator for Minecraft servers (Paper, Purpur, Spigot) and Java applications, built with Rust. Designed for low memory usage, fast processing, and readable error summaries.

This release provides both the **Minecraft Server Plugin** (`.jar`) for server panels like Pterodactyl and the **Standalone CLI Executable** for Linux and VPS environments.

---

## What is in this Release?

### 1. Paper / Spigot Minecraft Plugin (`LogMorph-0.1.0.jar`)
- Drop-in `.jar` for your server's `plugins/` directory.
- Works in **Pterodactyl**, Multicraft, and shared hosting panels without requiring SSH access.
- Native Rust engine embedded directly via safe JNI bindings.
- **Asynchronous Execution & Safe Lifecycle**: Log processing runs entirely on background workers and cancels gracefully on server stop (`onDisable()`), preventing tick lag or leaks.
- Commands supported:
  - `/logmorph` or `/lm`: Analyzes `logs/latest.log` and sends full colorized diagnostics to your console.
  - `/lm summary`: Prints condensed statistical summaries.
  - `/lm plugin <Name>`: Filters errors caused by a specific plugin.

### 2. Standalone Linux CLI (`logmorph-linux-x86_64`)
- Single self-contained binary for Linux x86_64 systems.
- Subcommands supported: `analyze`, `summary`, `inspect`, `watch`, and `export`.
- Standard process exit codes: `0` (Success), `1` (File Not Found), `2` (Invalid Input), `3` (Internal Error).
- Reads files directly (`logmorph logs/latest.log`) or via stdin pipe (`tail -f logs/latest.log | logmorph`).
- Memory ceiling control via `--max-signatures <N>` with first-seen retention.

### 3. Core Engine Capabilities
- **Smart Dynamic Normalization**: Selectively normalizes timestamps, UUIDs, memory addresses, and player coordinates, while preserving semantic numbers (HTTP status codes, network ports, versions, and SQL error codes).
- **Collision Protection**: 64-bit hash indexed with structural identity keys to ensure distinct errors are never mistakenly merged.
- **Attribution Classification**: Identifies root cause with explicit states: `confirmed`, `detected_from_stack_frame`, `ambiguous`, and `unknown`.
- **Framework Frame Filtering**: Automatically distinguishes between Minecraft framework internals (`net.minecraft`, `org.bukkit`, `com.destroystokyo.paper`) and plugin code to pinpoint root causes immediately.

---

## Release Assets & Checksums

| Asset File | Platform / Target | SHA-256 Checksum |
| :--- | :--- | :--- |
| `LogMorph-0.1.0.jar` | Minecraft Plugin (Paper / Spigot 1.20+) | `c8411852ce281a3e792747f5f54b792887d023a228e5936682cb0aab25650d96` |
| `logmorph-linux-x86_64` | Linux x86_64 Standalone Executable | `603e936a4f210c9785692a2aa6cb7a530036591b48613bfdc9f7ff35901f6e72` |

---

## Installation Quick Links

### For Minecraft Server Owners:
1. Download `LogMorph-0.1.0.jar`.
2. Place it in `plugins/` and restart your server.
3. Type `/lm` in your server console.

### For Linux Users:
```bash
curl -L -o logmorph https://github.com/Raine-oss/LogMorph/releases/download/v0.1.0/logmorph-linux-x86_64
chmod +x logmorph
sudo mv logmorph /usr/local/bin/
```

---

## Author & Contact

Maintained by **Raine-oss**.

- GitHub: [@Raine-oss](https://github.com/Raine-oss)
- Email: [rainebriars@gmail.com](mailto:rainebriars@gmail.com)
- Discord: [1478591836305231975](https://discord.com/users/1478591836305231975)
