# LogMorph v0.1.0 - Initial Release

A zero-overhead, streaming log analyzer and stack trace deduplicator built specifically for Minecraft servers (Paper, Purpur, Spigot) and Java applications.

This release provides both the **Minecraft Server Plugin** (`.jar`) for server panels like Pterodactyl and the **Standalone CLI Executable** for Linux and VPS environments.

---

## What is in this Release?

### 1. Paper / Spigot Minecraft Plugin (`LogMorph-0.1.0.jar`)
- Drop-in `.jar` for your server's `plugins/` directory.
- Works in **Pterodactyl**, Multicraft, and shared hosting panels without requiring SSH access.
- Native Rust engine embedded directly via safe JNI bindings.
- **Asynchronous execution**: Log processing runs entirely on background workers and will never drop ticks or freeze server gameplay.
- Commands supported:
  - `/logmorph` or `/lm`: Analyzes `logs/latest.log` and sends full colorized diagnostics to your console.
  - `/lm summary`: Prints condensed statistical summaries.
  - `/lm plugin <Name>`: Filters errors caused by a specific plugin.

### 2. Standalone Linux CLI (`logmorph-linux-x86_64`)
- Single self-contained binary for Linux x86_64 systems.
- Reads files directly (`logmorph logs/latest.log`) or via stdin pipe (`tail -f logs/latest.log | logmorph`).
- Constant memory footprint ($O(1)$ stream reading via `BufRead`).

### 3. Core Engine Capabilities
- **Deterministic Fingerprinting**: Generates clean 64-bit signatures based on the exception type and top plugin frame, ignoring dynamic numbers and timestamps.
- **Deduplication Engine**: Merges thousands of repeating stack traces into unique entries with occurrence counters and first/last seen timelines.
- **Framework Frame Filtering**: Automatically distinguishes between Minecraft framework internals (`net.minecraft`, `org.bukkit`, `com.destroystokyo.paper`) and plugin code to pinpoint root causes immediately.

---

## Release Assets & Checksums

| Asset File | Platform / Target | SHA-256 Checksum |
| :--- | :--- | :--- |
| `LogMorph-0.1.0.jar` | Minecraft Plugin (Paper / Spigot 1.20+) | `9e252ec51ad13b1421acbb9fe4666b2ea2bdc341b00309ca299d718107d33472` |
| `logmorph-linux-x86_64` | Linux x86_64 Standalone Executable | `35283b4e736537673ed0173f01e71983585920bf09dfdfa07e057ada52909276` |

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
