# File Integrity Monitor (FIM)

A production-quality, Linux-based File Integrity Monitoring (FIM) system written in Rust. It recursively scans directories, computes SHA256 hashes of files, stores them in a MySQL/MariaDB database, and detects file changes across scans.

## Features

- **Efficient File Scanning**: Uses `walkdir` crate for recursive directory traversal
- **Secure Hashing**: SHA256 hashing with buffered file reading for memory efficiency
- **Database Persistence**: MySQL/MariaDB backend for storing file metadata
- **Change Detection**: Automatically detects new, modified, and deleted files
- **Systemd Integration**: Timer-based scheduling for regular scans
- **Comprehensive Logging**: Both console and file-based logging support
- **Error Resilience**: Gracefully handles file permission errors and DB connectivity issues
- **CLI Arguments**: Configurable scan directory and log output paths

## Prerequisites

- **Rust**: 1.56+ (install from https://rustup.rs/)
- **MySQL/MariaDB**: 5.7+ (or equivalent)
- **Linux**: Arch Linux, EndeavourOS, Debian, RHEL, or any systemd-based distribution

## Installation

### 1. Clone or Extract the Project

```bash
cd /opt/fim  # Recommended installation location
cd cysec-proj  # Your project directory
```

### 2. Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Configure Environment Variables

Copy the example `.env` file and update with your database credentials:

```bash
cp .env.example .env
nano .env
```

**Required environment variables:**

```
DB_HOST=localhost       # MySQL server hostname
DB_USER=fim_user        # MySQL database user
DB_PASS=secure_password # MySQL user password
DB_NAME=fim_database    # Database name
```

### 4. Create MySQL Database and User

```bash
sudo mysql -u root -p
```

```sql
CREATE DATABASE fim_database;
CREATE USER 'fim_user'@'localhost' IDENTIFIED BY 'secure_password';
GRANT ALL PRIVILEGES ON fim_database.* TO 'fim_user'@'localhost';
FLUSH PRIVILEGES;
EXIT;
```

**Note**: The application automatically creates the `files` table on first run.

### 5. Build the Project

```bash
cargo build --release
```

The compiled binary will be at `target/release/fim`.

## Usage

### Basic Usage

Scan the current directory and store results in the database:

```bash
./target/release/fim .
```

Scan a specific directory:

```bash
sudo ./target/release/fim /etc
```

Scan and log results to a file:

```bash
sudo ./target/release/fim /etc --log /var/log/fim.log
```

### Using the Provided Script

The `script.sh` wrapper handles environment loading and provides a convenient interface:

```bash
# Make script executable
chmod +x script.sh

# Scan current directory
./script.sh .

# Scan /etc (requires sudo for some files)
sudo ./script.sh /etc

# The script automatically logs to /tmp/fim-*.log if /var/log is not writable
```

## Systemd Integration

For automated, scheduled scans using systemd timers:

### 1. Install Files

```bash
# Copy service and timer files
sudo cp fim.service /etc/systemd/system/
sudo cp fim.timer /etc/systemd/system/

# Update paths in the service file if needed
sudo nano /etc/systemd/system/fim.service
```

Update `ExecStart` and `WorkingDirectory` to match your installation path:

```ini
WorkingDirectory=/opt/fim
ExecStart=/opt/fim/script.sh /etc
```

### 2. Enable and Start the Timer

```bash
# Reload systemd daemon
sudo systemctl daemon-reload

# Enable timer to start on boot
sudo systemctl enable fim.timer

# Start the timer immediately
sudo systemctl start fim.timer

# Verify timer status
sudo systemctl status fim.timer

# View timer schedule
sudo systemctl list-timers fim.timer

# Check service logs
sudo journalctl -u fim.service -f
```

### 3. Manual Service Invocation

Run the service manually without waiting for the timer:

```bash
sudo systemctl start fim.service
```

### 4. View Logs

```bash
# View all FIM logs
sudo journalctl -u fim.service -n 50

# Follow logs in real-time
sudo journalctl -u fim.service -f

# View logs since last boot
sudo journalctl -u fim.service --since today
```

## Configuration

### Adjusting Timer Schedule

Edit the timer to change the scan schedule:

```bash
sudo nano /etc/systemd/system/fim.timer
```

**Common schedule options:**

```ini
# Daily at 2:00 AM
OnCalendar=*-*-* 02:00:00

# Every 6 hours
OnUnitActiveSec=6h

# Every Monday at 3:00 AM
OnCalendar=Mon *-*-* 03:00:00

# First day of month at midnight
OnCalendar=*-*-01 00:00:00
```

After changes, reload and restart:

```bash
sudo systemctl daemon-reload
sudo systemctl restart fim.timer
```

### Adjusting Scan Directory

Edit the service file to change the directory being scanned:

```bash
sudo nano /etc/systemd/system/fim.service
```

Change the `ExecStart` line:

```ini
ExecStart=/opt/fim/script.sh /path/to/scan
```

## Database Schema

The application automatically creates the following table:

```sql
CREATE TABLE files (
    path VARCHAR(4096) PRIMARY KEY,
    hash VARCHAR(64) NOT NULL,
    last_checked TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### Querying Results

```sql
-- All files
SELECT * FROM files;

-- Files modified in the last hour
SELECT * FROM files WHERE last_checked >= DATE_SUB(NOW(), INTERVAL 1 HOUR);

-- Count of files
SELECT COUNT(*) FROM files;

-- Find specific file
SELECT * FROM files WHERE path LIKE '%/etc/passwd%';
```

## Output Format

### Console Output

```
[INFO] Starting File Integrity Monitor
[INFO] Scanning directory: /etc
[INFO] Found 1234 files
[INFO] Computing hashes...
[INFO] Connected to database
[NEW] /etc/newfile.conf
[MODIFIED] /etc/passwd (old: abc123..., new: def456...)
[DELETED] /etc/oldfile.conf
[SUMMARY] Unchanged: 1200, New: 2, Modified: 1, Deleted: 1
```

### Log File Format

When using `--log` option, results are appended to the log file with timestamps:

```
--- 2024-04-08 14:30:45 ---
[NEW] /etc/newfile.conf
[MODIFIED] /etc/passwd (old: abc123..., new: def456...)
[DELETED] /etc/oldfile.conf
```

## Error Handling

The application gracefully handles common errors:

- **Unreadable files**: Logs warning and continues scanning
- **Database connection failures**: Exits with error message
- **Missing directories**: Reports error and exits
- **Permission denied**: Logs warning, continues processing
- **File changes during scan**: Detected on next run

## Performance Considerations

- **Buffered reading**: Files are read in 8KB chunks to minimize memory usage
- **Efficient hashing**: Uses optimized SHA256 implementation
- **Database optimization**: Batch operations where possible
- **Skip non-files**: Directories are automatically skipped

### Performance Tips

- Run scans during off-peak hours
- For large directories (>100k files), consider splitting into multiple scans
- Use SSD storage for the database for better performance
- Monitor system resources during large scans

## Troubleshooting

### "Failed to connect to database"

```bash
# Check MySQL is running
sudo systemctl status mysql
sudo systemctl status mariadb

# Verify credentials in .env
cat .env

# Test connection manually
mysql -h localhost -u fim_user -p
```

### "Permission denied" errors

Ensure you run scans with appropriate permissions:

```bash
# Scan user directories
./target/release/fim /home/user

# Scan system directories (requires sudo)
sudo ./target/release/fim /etc

# Check file permissions
ls -la /path/to/file
```

### Timer not running

```bash
# Check timer status
sudo systemctl status fim.timer

# Check for errors in systemd journal
sudo journalctl -xe | grep fim

# Verify service file syntax
systemd-analyze verify /etc/systemd/system/fim.service
```

### Database table not created

The table is created automatically. If it's missing:

```bash
# Check database exists
mysql -u fim_user -p -e "SHOW DATABASES;"

# Check table exists
mysql -u fim_user -p fim_database -e "SHOW TABLES;"

# If missing, run the application once
./target/release/fim . 2>&1 | head -20
```

## Security Considerations

- **Database Credentials**: Store `.env` with restricted permissions
  ```bash
  chmod 600 .env
  ```
- **Systemd Service**: Consider using systemd security features
  ```ini
  [Service]
  PrivateDevices=yes
  ProtectSystem=strict
  ProtectHome=yes
  ```
- **Database Access**: Use strong passwords and limit user privileges
- **Scan Permissions**: Run with minimal required privileges

## Development

### Run Tests

```bash
cargo test
```

### Build Debug Binary

```bash
cargo build

# Binary at target/debug/fim
./target/debug/fim .
```

### Enable Verbose Logging

```bash
RUST_LOG=debug ./target/release/fim .
```

## Architecture

```
src/
├── main.rs          # CLI entry point and orchestration
├── scanner.rs       # Directory scanning with walkdir
├── hasher.rs        # SHA256 hash computation
├── db.rs            # MySQL database operations
└── comparator.rs    # Hash comparison and change detection
```

### Module Overview

- **scanner.rs**: Recursively finds all files in a directory
- **hasher.rs**: Computes SHA256 hashes efficiently with buffered I/O
- **db.rs**: Manages database connections and CRUD operations
- **comparator.rs**: Compares current state with stored state to detect changes

## Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test scanner::
cargo test hasher::
cargo test comparator::

# Run with output
cargo test -- --nocapture
```

## License

This project is provided as-is for system integrity monitoring purposes.

## Support

For issues or questions:

1. Check the Troubleshooting section
2. Review application logs with `journalctl`
3. Verify MySQL connectivity
4. Ensure proper file permissions

## Changelog

### v0.1.0 - Initial Release

- Complete FIM implementation
- MySQL database backend
- Systemd timer integration
- SHA256 hashing with buffered I/O
- Comprehensive error handling
- CLI argument support
- File and console logging

