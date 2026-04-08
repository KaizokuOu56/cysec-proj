# Changelog

All notable changes to the File Integrity Monitor (FIM) project will be documented in this file.

## [0.1.0] - 2024-04-08

### Added

- Initial release of File Integrity Monitor (FIM)
- **Core Functionality**:
  - Recursive directory scanning with `walkdir` crate
  - SHA256 hashing with buffered file I/O (8KB chunks)
  - MySQL/MariaDB database backend for file metadata persistence
  - File change detection (new, modified, deleted files)
  - Comprehensive error handling and graceful failure recovery

- **Architecture**:
  - Modular code organization:
    - `scanner.rs`: Directory traversal with safe error handling
    - `hasher.rs`: Efficient file hashing with memory-safe buffering
    - `db.rs`: Database abstraction layer with connection pooling
    - `comparator.rs`: Change detection algorithm with O(1) lookups
    - `main.rs`: CLI interface and orchestration

- **CLI Features**:
  - Configurable directory scanning: `fim <directory>`
  - Optional file logging: `fim <directory> --log <logfile>`
  - Help system: `fim --help`

- **Database**:
  - Automatic table creation on first run
  - Stores: file path, SHA256 hash, last checked timestamp
  - Supports any MySQL 5.7+ / MariaDB server

- **Integration**:
  - Systemd service file (`fim.service`) for execution
  - Systemd timer file (`fim.timer`) for scheduling (default: 2:00 AM daily)
  - Bash wrapper script (`script.sh`) for environment loading
  - Environment configuration via `.env` file

- **Quality**:
  - 10 comprehensive unit tests (all passing)
  - Zero unsafe code
  - Production-ready error handling
  - Idiomatic Rust patterns throughout

- **Documentation**:
  - Comprehensive README with:
    - Installation instructions
    - Database setup guide
    - Systemd integration examples
    - Troubleshooting section
    - Architecture overview
    - Security considerations
    - Performance tuning tips

- **Performance**:
  - Buffered file reading to minimize memory usage
  - Database connection pooling for efficiency
  - Fast hash computation with optimized SHA256
  - O(1) file lookup for change detection

### Technical Stack

- **Language**: Rust (stable, 1.56+)
- **Key Dependencies**:
  - `walkdir` (2.x) - Efficient directory traversal
  - `sha2` (0.10) - SHA256 hashing
  - `hex` (0.4) - Hex encoding for hashes
  - `mysql` (25.x) - MySQL/MariaDB driver with connection pooling
  - `clap` (4.x) - CLI argument parsing
  - `dotenvy` (0.15) - Environment variable loading
  - `chrono` (0.4) - Timestamp handling
  - `tracing` (0.1) - Logging infrastructure

### Known Limitations

1. Requires pre-existing MySQL/MariaDB server (not bundled)
2. Requires `.env` file with database credentials
3. Currently scans entire directory tree (no exclusion filters)
4. Synchronous implementation (no async I/O)

### Installation & Usage

```bash
# Build release binary
cargo build --release

# Run default scan
./target/release/fim /path/to/scan

# With logging
./target/release/fim /path/to/scan --log /var/log/fim.log

# Set up systemd timer
sudo cp fim.service fim.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now fim.timer
```

See README.md for detailed setup instructions.

