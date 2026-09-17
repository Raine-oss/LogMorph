# Contributing to LogMorph

Thank you for your interest in contributing to LogMorph. We welcome bug reports, feature requests, documentation improvements, and code contributions.

---

## Code of Conduct

Please be respectful, constructive, and helpful when interacting with other contributors and maintainers.

---

## Development Setup

LogMorph consists of:
1. **Rust Core & CLI**: Requires Rust stable (`rustc` and `cargo`).
2. **Minecraft Paper/Spigot Plugin**: Requires Java 17+ and Maven 3.8+.

### Prerequisites
- Rust 1.75+
- OpenJDK 17 or 21
- Maven 3.8+

### Building the Project
```bash
# Clone the repository
git clone https://github.com/Raine-oss/LogMorph.git
cd LogMorph

# Build Rust Core & CLI
cargo build

# Run Rust unit and integration tests
cargo test

# Build Paper/Spigot Plugin
mkdir -p plugin/src/main/resources/natives/linux-x86_64
cp target/debug/liblogmorph.so plugin/src/main/resources/natives/linux-x86_64/
cd plugin
mvn test
mvn clean package
```

---

## Code Standards

- **Rust**: Follow standard Rust formatting (`cargo fmt`) and linting (`cargo clippy`).
- **Comments**: Keep comments minimal and structured strictly as section headers (`// Header`). Do not add tutorial or explanatory comments.
- **Safety**: Never let panics cross the JNI boundary. All native functions called by Java must use `std::panic::catch_unwind`.
- **Async Execution**: The Minecraft plugin must never execute heavy file reading or native calls on the main tick thread.

---

## Submitting Pull Requests

1. Fork the repository and create a new feature branch:
   ```bash
   git checkout -b feature/my-feature
   ```
2. Commit your changes with clear, conventional commit messages (`feat: ...`, `fix: ...`, `docs: ...`).
3. Ensure all tests pass before opening a PR:
   ```bash
   cargo test
   cd plugin && mvn test
   ```
4. Push to your fork and submit a Pull Request to `main`.
