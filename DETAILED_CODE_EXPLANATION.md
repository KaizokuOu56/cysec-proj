# Complete Code Explanation: File Integrity Monitor

**For Developers Familiar with Programming Concepts but New to Rust, Bash, and Systemd**

---

## Table of Contents

1. [What Does This Project Do?](#what-does-this-project-do)
2. [Core Programming Concepts in This Codebase](#core-programming-concepts)
3. [External Dependencies (Libraries/Tools)](#external-dependencies)
4. [Source Code Files](#source-code-files)
5. [Configuration and Deployment Files](#configuration-and-deployment-files)
6. [Module Interaction Model](#module-interaction-model)
7. [Execution Workflow](#execution-workflow)
8. [Data Transformation Pipeline](#data-transformation-pipeline)

---

## What Does This Project Do?

This is a File Integrity Monitor (FIM) system that implements a change detection mechanism based on cryptographic hashing. The system operates on three core principles:

1. **Baseline Creation**: Computes SHA256 hashes of all files within a specified directory tree, creating a cryptographic fingerprint for each file's current state
2. **Persistent Storage**: Stores these hashes alongside file paths and timestamps in a relational database (MySQL/MariaDB) for historical comparison
3. **Change Detection**: On subsequent runs, recomputes hashes and compares them against stored baseline values to identify modifications, additions, and deletions

This approach allows detection of unauthorized or unexpected file modifications with cryptographic certainty—any change to a file's content, however small, produces a different hash value. The system maintains an audit trail of changes over time by updating database records with new timestamps when modifications are detected.

---

## Core Programming Concepts in This Codebase

### Functions
Functions in this codebase are procedural abstractions that encapsulate specific responsibilities within the system. Each function accepts parameters (inputs), executes a sequence of operations, and returns a Result type indicating either successful completion with a value or failure with an error. The Rust convention of returning `Result<T, E>` is used throughout to enable compile-time verification of error handling.

### Ownership and Borrowing (Rust-Specific)
Rust enforces strict memory safety through its ownership system. Variables own their data until explicitly transferred (moved). The `&` operator creates a borrowed reference, allowing functions to access data without taking ownership. The `mut` keyword indicates a binding is mutable. This prevents common memory errors at compile time.

### Structs and Enums
Data structures are defined using `struct` for product types (multiple named fields) and `enum` for sum types (mutually exclusive variants). `struct FileRecord` groups related fields; `enum FileStatus` represents one of four possible states. The `impl` keyword attaches methods to these types.

### Error Handling via Result Types
Rather than exceptions, this codebase uses the `Result<T, E>` type. `Ok(value)` represents success; `Err(error)` represents failure. The `?` operator provides syntactic sugar for propagating errors up the call stack, similar to exception throwing in other languages.

### Generics and Traits
Generic functions and types are used extensively (e.g., `Vec<T>`). Traits define shared behavior across types. The `impl` keyword is used to implement traits for specific types. The `dyn` keyword enables dynamic dispatch for trait objects.

---

## External Dependencies (Libraries/Tools)

The project uses the following external crates (Rust libraries):

### **walkdir** - Recursive Directory Traversal
Provides efficient recursive directory traversal with automatic handling of symbolic links and permission errors. The `WalkDir::new()` constructor returns an iterator over directory entries. The iterator yields `DirEntry` objects containing filesystem metadata accessible via methods like `is_file()`, `metadata()`, and `path()`.

### **sha2** - Cryptographic Hashing
Implements the SHA-2 family of cryptographic hash functions. The `Sha256::new()` constructor creates a new hash context. The `update()` method adds data to the hash computation via a streaming interface (suitable for large files). The `finalize()` method produces the final 256-bit hash output. This is memory-efficient as it processes data in chunks rather than requiring the entire file in memory.

### **hex** - Hexadecimal Encoding  
Converts binary data into human-readable hexadecimal notation. The `hex::encode()` function transforms raw hash bytes into a 64-character hexadecimal string suitable for storage and display.

### **mysql** - Database Client Library
Provides a synchronous MySQL/MariaDB client implementation. The library includes:
- `Pool`: Connection pooling for efficient reuse of database connections
- `OptsBuilder`: Fluent builder pattern for constructing connection options
- Prepared statement support to prevent SQL injection and improve performance

### **clap** - Command-Line Argument Parser
Derives a command-line interface from struct definitions using procedural macros. The `#[derive(Parser)]` macro generates parsing code. The `#[command(...)]` and `#[arg(...)]` attributes configure parsing behavior. This eliminates manual argument parsing overhead.

### **dotenvy** - Environment Variable Loading
Reads `.env` files and loads key-value pairs into the process environment. Useful for managing configuration without hardcoding secrets or configuration values. The `.ok()` method suppresses errors if the file doesn't exist.

### **chrono** - Date/Time Handling
Provides timezone-aware date and time manipulation. `Utc::now()` returns the current UTC time. The `.format()` method produces formatted string output compatible with database timestamp columns.

### **tracing/tracing-subscriber** - Structured Logging
Provides structured logging with configurable filters. `tracing_subscriber::fmt()` configures console output. The `.with_env_filter()` method allows runtime filtering via the `RUST_LOG` environment variable.

---

## Source Code Files

### File 1: `Cargo.toml` - The Project Blueprint

This file tells Rust how to build the project. It's like a recipe card for building software.

```toml
[package]
name = "file-integrity-monitor"
```
**Explanation**: The project's name is "file-integrity-monitor". This is the official name.

```toml
version = "0.1.0"
```
**Explanation**: This is version 0.1.0 (first version, first release).

```toml
edition = "2021"
```
**Explanation**: This program uses Rust from 2021. (Rust updates annually; this uses 2021's features.)

```toml
[dependencies]
walkdir = "2"
sha2 = "0.10"
hex = "0.4"
mysql = "25"
clap = { version = "4", features = ["derive"] }
dotenvy = "0.15"
chrono = "0.4"
```
**Explanation**: These are the external libraries we're using. Each line says:
- We need `walkdir` version 2 (to walk directories)
- We need `sha2` version 0.10 (to create hashes)
- We need `hex` version 0.4 (to convert hashes to text)
- We need `mysql` version 25 (to talk to database)
- And so on...

```toml
[dev-dependencies]
tempfile = "3"
```
**Explanation**: This library is only used for testing. It creates temporary files for tests.

---

### File 2: `src/scanner.rs` - Finding All Files

**Purpose**: This file's job is to find all files in a directory (including all subfolders).

Let me explain every line:

#### Lines 1-2: Importing Tools
```rust
use std::path::{Path, PathBuf};
use walkdir::{WalkDir, DirEntry};
```

**Explanation**: 
- `use` means "import" or "borrow someone else's code"
- `std::path::{Path, PathBuf}` - We're borrowing file path handling tools from Rust's built-in library
- `Path` = A way to refer to a file path (like "/home/user/file.txt")
- `PathBuf` = A file path that we can actually store and modify
- `WalkDir` = The tool from the `walkdir` library that walks through directories
- `DirEntry` = Information about one file or folder that we found

#### Lines 4-6: Creating a Data Structure
```rust
#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub path: PathBuf,
}
```

**Explanation**:
- `struct ScannedFile {` = We're creating a data structure called "ScannedFile"
- This structure holds information about one file that we found
- `pub path: PathBuf;` = Each ScannedFile has a member called `path` that stores the file's location
- `pub` means "public" — anyone who imports this can use it
- `PathBuf` is the type — it must be a file path
- `#[derive(Debug, Clone)]` = Special instructions: allow debugging output and copying this structure

#### Lines 9-11: Function Declaration
```rust
pub fn scan_directory(root: &Path) -> Result<Vec<ScannedFile>, Box<dyn std::error::Error>> {
```

**Explanation**:
- `pub fn scan_directory` = We're creating a public function named "scan_directory"
- `(root: &Path)` = This function takes one input: a file path called "root" (the directory to search)
- `&Path` = The `&` means "reference" (like pointing at something instead of copying it)
- `-> Result<Vec<ScannedFile>, Box<dyn std::error::Error>>` = What the function returns:
  - `Result` = Either success or failure
  - `Vec<ScannedFile>` = If success: a list of ScannedFile objects
  - `Box<dyn std::error::Error>` = If failure: an error message

#### Lines 12-16: Checking the Directory Exists
```rust
if !root.exists() {
    return Err(format!("Directory does not exist: {}", root.display()).into());
}

if !root.is_dir() {
    return Err(format!("Path is not a directory: {}", root.display()).into());
}
```

**Explanation**:
- `if !root.exists()` = If the directory does NOT exist (`!` means "not")
- `return` = Stop the function immediately and send back a result
- `Err(...)` = Send back an error (bad result)
- `format!(...)` = Create an error message
- Same for the second check: make sure it's actually a directory, not a file

#### Lines 18-19: Creating an Empty List
```rust
let mut files = Vec::new();
```

**Explanation**:
- `let` = Create a new variable
- `mut` = "mutable" (we can change it later)
- `files` = The variable name
- `Vec::new()` = Create an empty list (Vec = Vector = list)
- We'll fill this list with all the files we find

#### Lines 21-31: The Main Loop (Finding Files)
```rust
for entry in WalkDir::new(root)
    .into_iter()
    .filter_map(|e| {
        if let Err(err) = &e {
            eprintln!("[WARN] Error reading directory entry: {}", err);
            None
        } else {
            Some(e.ok().unwrap())
        }
    })
```

**Explanation**:
- `for entry in WalkDir::new(root)` = Start walking through the directory starting at "root"
- `.into_iter()` = Convert the results into something we can loop through
- `.filter_map(|e| { ... })` = For each entry, check if it's good or bad
  - `|e|` = The entry itself (like a parameter)
  - `if let Err(err) = &e` = If there's an error (like "permission denied")
  - `eprintln!(...)` = Print an error message to the screen
  - `None` = Skip this entry (don't include it)
  - `else { Some(e.ok().unwrap()) }` = If no error, include this entry

#### Lines 33-43: Processing Each Entry
```rust
if entry.file_type().is_file() {
    if let Ok(metadata) = entry.metadata() {
        if metadata.is_file() {
            let path = entry.path().to_path_buf();
            files.push(ScannedFile { path });
        }
    }
}
```

**Explanation**:
- `if entry.file_type().is_file()` = Check if this entry is a file (not a directory)
- `if let Ok(metadata) = entry.metadata()` = Try to get information about this file
  - `Ok(...)` = Success case
  - `metadata` = Information about the file (size, permissions, etc.)
- `if metadata.is_file()` = Double-check it's really a file
- `let path = entry.path().to_path_buf();` = Get the file's path and convert it to PathBuf format
- `files.push(ScannedFile { path });` = Add this file to our list
  - `push` = Add to the end of the list
  - `ScannedFile { path }` = Create a new ScannedFile object with this path

#### Lines 45-46: Return Success
```rust
Ok(files)
```

**Explanation**:
- Return success with the list of files we found
- `Ok` = Success (as opposed to `Err` which is failure)

#### Lines 48-73: Test Code
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let files = scan_directory(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 0);
    }
```

**Explanation**:
- `#[cfg(test)]` = This section only runs when testing
- `mod tests` = Create a module (section) for tests
- `fn test_scan_empty_directory()` = Test function: "Does scanning an empty directory return zero files?"
- `let temp_dir = TempDir::new().unwrap();` = Create a temporary directory for testing
- `let files = scan_directory(temp_dir.path()).unwrap();` = Scan it and get the list
- `assert_eq!(files.len(), 0);` = Check that the length is 0 (it's empty, so this should be true)

**The other tests**: Similar logic
- Test with 2 files: Does it find exactly 2 files?
- Test nonexistent directory: Does it return an error?

---

### File 3: `src/hasher.rs` - Creating Fingerprints

**Purpose**: This file's job is to create a unique fingerprint (hash) for each file by reading its contents.

#### Lines 1-4: Importing Tools
```rust
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
```

**Explanation**:
- `sha2::{Sha256, Digest}` = Import the SHA256 hashing algorithm and the Digest interface
- `std::fs::File` = Tool for reading files
- `std::io::{self, Read}` = Tools for reading data
- `std::path::Path` = Tool for file paths

#### Lines 6: Setting Buffer Size
```rust
const BUFFER_SIZE: usize = 8192;
```

**Explanation**:
- `const` = This is a constant (never changes)
- `BUFFER_SIZE` = The name
- `= 8192` = 8,192 bytes (about 8 kilobytes)
- We'll read files in 8KB chunks (not loading the entire file at once)
- This saves memory for huge files

#### Lines 10-11: Function Declaration
```rust
pub fn compute_file_hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
```

**Explanation**:
- `pub fn compute_file_hash` = Public function to compute a file's hash
- `(path: &Path)` = Takes a file path as input
- `-> Result<String, ...>` = Returns either:
  - `String` = Success: the hash as text (like "abc123def456...")
  - Error information = Failure (couldn't read file)

#### Lines 12-13: Opening the File
```rust
let mut file = File::open(path)?;
let mut hasher = Sha256::new();
```

**Explanation**:
- `let mut file = File::open(path)?;` = Try to open the file
  - `?` = If this fails, immediately return an error (stop here)
  - `mut` = We'll be reading from this file
- `let mut hasher = Sha256::new();` = Create a new SHA256 hasher
  - Think of it as an empty container ready to receive data

#### Line 14: Creating Buffer
```rust
let mut buffer = [0; BUFFER_SIZE];
```

**Explanation**:
- `let mut buffer = [0; BUFFER_SIZE];` = Create an array (list) of 8,192 zeros
- We'll fill this with chunks of the file

#### Lines 16-21: The Reading Loop
```rust
loop {
    let bytes_read = file.read(&mut buffer)?;
    if bytes_read == 0 {
        break;
    }
    hasher.update(&buffer[..bytes_read]);
}
```

**Explanation**:
- `loop {` = Repeat forever (until we break out)
- `let bytes_read = file.read(&mut buffer)?;` = Read up to 8,192 bytes from the file into the buffer
  - `bytes_read` = How many bytes we actually read (might be less than 8,192 at the end)
  - `?` = If error, stop and return the error
- `if bytes_read == 0` = If we read zero bytes, we've reached the end of the file
- `break;` = Exit the loop
- `hasher.update(&buffer[..bytes_read]);` = Feed these bytes to the hasher
  - `&buffer[..bytes_read]` = Only the part we actually read

#### Lines 23-25: Creating the Final Hash
```rust
let hash = hasher.finalize();
Ok(hex::encode(hash))
```

**Explanation**:
- `let hash = hasher.finalize();` = Complete the hashing process and get the result
- `hex::encode(hash)` = Convert the hash to readable text (hexadecimal format)
- `Ok(...)` = Return success with the hash string

#### Lines 27-53: Tests
Similar to scanner tests, but for hashing:
- Test empty file: Does it produce the correct hash?
- Test consistency: Does the same file always produce the same hash?
- Test missing file: Does it return an error for nonexistent files?

---

### File 4: `src/db.rs` - Database Operations

**Purpose**: This file manages all communication with the MySQL/MariaDB database where we store file hashes.

#### Lines 1-3: Importing Tools
```rust
use mysql::prelude::*;
use mysql::{Pool, OptsBuilder, Result as MysqlResult};
use chrono::Utc;
```

**Explanation**:
- `mysql::prelude::*` = Import all the basic MySQL tools
- `Pool` = Connection pool (keeps multiple database connections ready)
- `OptsBuilder` = Tool to build database connection options
- `chrono::Utc` = Tool to get current date/time in UTC

#### Lines 5-9: Data Structure for File Record
```rust
#[derive(Debug, Clone)]
pub struct FileRecord {
    pub path: String,
    pub hash: String,
    pub last_checked: String,
}
```

**Explanation**:
- This represents one row in the database
- `path` = Where the file is located (like "/home/user/file.txt")
- `hash` = The file's SHA256 hash
- `last_checked` = When we last computed this hash

#### Lines 11-13: Database Struct
```rust
pub struct Database {
    pool: Pool,
}
```

**Explanation**:
- This represents our connection to the database
- `pool` = The connection pool (multiple connections ready to use)

#### Lines 15-31: Creating Database Connection
```rust
pub fn new(
    host: &str,
    user: &str,
    password: &str,
    database: &str,
) -> Result<Self, Box<dyn std::error::Error>> {
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(host))
        .user(Some(user))
        .pass(Some(password))
        .db_name(Some(database));
    
    let pool = Pool::new(opts)?;
```

**Explanation**:
- `pub fn new(...)` = Function to create a new Database connection
- Takes 4 parameters: host (server address), user, password, database name
- `OptsBuilder::new()` = Create connection options
- `.ip_or_hostname(Some(host))` = Set the server address
- `.user(Some(user))` = Set the username
- `.pass(Some(password))` = Set the password
- `.db_name(Some(database))` = Set which database to use
- `Pool::new(opts)?` = Create a connection pool with these options

#### Lines 32-42: Creating the Table
```rust
let mut conn = pool.get_conn()?;
conn.exec_drop(
    "CREATE TABLE IF NOT EXISTS files (
        path VARCHAR(4096) PRIMARY KEY,
        hash VARCHAR(64) NOT NULL,
        last_checked TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
    )",
    (),
)?;

Ok(Database { pool })
```

**Explanation**:
- `let mut conn = pool.get_conn()?;` = Get a connection from the pool
- `conn.exec_drop(...)` = Execute a SQL command (SQL is database language)
- `CREATE TABLE IF NOT EXISTS files` = Create a table called "files" if it doesn't exist
  - `path VARCHAR(4096) PRIMARY KEY` = Column: file path (unique identifier)
  - `hash VARCHAR(64)` = Column: the 64-character hash
  - `last_checked TIMESTAMP` = Column: when it was last checked
- `Ok(Database { pool })` = Return success with the Database struct

#### Lines 45-53: Getting All Files
```rust
pub fn get_all_files(&self) -> MysqlResult<Vec<FileRecord>> {
    let mut conn = self.pool.get_conn()?;
    conn.query_map(
        "SELECT path, hash, last_checked FROM files",
        |(path, hash, last_checked)| FileRecord {
            path,
            hash,
            last_checked,
        },
    )
}
```

**Explanation**:
- `pub fn get_all_files(&self)` = Function to get all records from the database
- `&self` = Use this Database object
- `conn.query_map(...)` = Query the database and convert results
- `"SELECT path, hash, last_checked FROM files"` = SQL: Get these three columns from files table
- `|(path, hash, last_checked)| FileRecord { ... }` = For each row, create a FileRecord object

#### Lines 56-70: Inserting a New Record
```rust
pub fn insert_file(&self, path: &str, hash: &str) -> MysqlResult<()> {
    let mut conn = self.pool.get_conn()?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.exec_drop(
        "INSERT INTO files (path, hash, last_checked) VALUES (?, ?, ?)",
        (path, hash, now),
    )?;
    Ok(())
}
```

**Explanation**:
- `pub fn insert_file(...)` = Function to add a new file record
- `let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();` = Get current time as formatted text
- `INSERT INTO files ...` = SQL: Add a new row to the files table
- `VALUES (?, ?, ?)` = Three values (placeholders)
- `(path, hash, now)` = The actual values to insert

#### Lines 72-80: Updating a Record
```rust
pub fn update_file(&self, path: &str, hash: &str) -> MysqlResult<()> {
    let mut conn = self.pool.get_conn()?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.exec_drop(
        "UPDATE files SET hash = ?, last_checked = ? WHERE path = ?",
        (hash, now, path),
    )?;
    Ok(())
}
```

**Explanation**:
- `pub fn update_file(...)` = Function to update an existing record
- `UPDATE files SET` = SQL: Modify rows in the files table
- `hash = ?` = Change the hash to this value
- `last_checked = ?` = Change the timestamp to now
- `WHERE path = ?` = Only for this specific file path

#### Lines 82-88: Deleting a Record
```rust
pub fn delete_file(&self, path: &str) -> MysqlResult<()> {
    let mut conn = self.pool.get_conn()?;
    conn.exec_drop("DELETE FROM files WHERE path = ?", (path,))?;
    Ok(())
}
```

**Explanation**:
- `pub fn delete_file(...)` = Function to delete a record
- `DELETE FROM files WHERE path = ?` = SQL: Remove the row with this path

---

### File 5: `src/comparator.rs` - Detecting Changes

**Purpose**: This file compares old file hashes (from database) with new hashes (just computed) to detect changes.

#### Lines 1: Importing Tools
```rust
use std::collections::HashSet;
```

**Explanation**:
- `HashSet` = A set (collection) that's optimized for fast lookups
- Think of it like a card catalog in a library — fast to find specific items

#### Lines 3-7: Status Enumeration
```rust
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FileStatus {
    New,
    Modified,
    Unchanged,
    Deleted,
}
```

**Explanation**:
- `enum FileStatus` = A type that can be one of four values:
  - `New` = File didn't exist in database, it's new
  - `Modified` = File exists but hash changed (modified)
  - `Unchanged` = File exists and hash is the same
  - `Deleted` = File was in database but now missing

#### Lines 9-15: File Change Structure
```rust
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub status: FileStatus,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
}
```

**Explanation**:
- `pub struct FileChange` = Represents one file and what changed about it
- `path` = The file's path
- `status` = Is it New, Modified, Unchanged, or Deleted?
- `old_hash` = The previous hash (might be None if it's new)
- `new_hash` = The current hash (might be None if it's deleted)
- `Option<String>` = "Maybe a string, or maybe nothing"

#### Lines 18-25: Comparison Result Structure
```rust
#[derive(Debug)]
pub struct ComparisonResult {
    pub new_files: Vec<FileChange>,
    pub modified_files: Vec<FileChange>,
    pub deleted_files: Vec<FileChange>,
    pub unchanged_count: usize,
}
```

**Explanation**:
- `pub struct ComparisonResult` = The final results of comparing old vs new hashes
- `new_files` = List of newly discovered files
- `modified_files` = List of changed files
- `deleted_files` = List of files that were removed
- `unchanged_count` = How many files didn't change (just a count)

#### Lines 28-31: Results Helper Method
```rust
impl ComparisonResult {
    pub fn total_changes(&self) -> usize {
        self.new_files.len() + self.modified_files.len() + self.deleted_files.len()
    }
}
```

**Explanation**:
- `impl ComparisonResult` = Add a method to ComparisonResult
- `pub fn total_changes(&self)` = Function to calculate total number of changes
- Adds the lengths of all three lists

#### Lines 34-38: Function Declaration
```rust
pub fn compare_hashes(
    current_files: &[(String, String)],
    database_files: &[(String, String)],
) -> ComparisonResult {
```

**Explanation**:
- `pub fn compare_hashes` = Public function to compare files
- `current_files: &[(String, String)]` = Input 1: list of (path, hash) pairs just scanned
- `database_files: &[(String, String)]` = Input 2: list of (path, hash) pairs from database
- `-> ComparisonResult` = Returns a ComparisonResult with all the changes

#### Lines 39-42: Initialize Result Lists
```rust
let mut new_files = Vec::new();
let mut modified_files = Vec::new();
let mut deleted_files = Vec::new();
let mut unchanged_count = 0;
```

**Explanation**:
- Create empty lists for each category of changes
- Initialize counter for unchanged files to 0

#### Lines 45-47: Create Lookup Structures
```rust
let db_map: std::collections::HashMap<_, _> = database_files.iter().cloned().collect();
let current_set: HashSet<_> = current_files.iter().map(|(p, _)| p.clone()).collect();
let _db_set: HashSet<_> = database_files.iter().map(|(p, _)| p.clone()).collect();
```

**Explanation**:
- `db_map` = Convert database files into a HashMap for fast lookups
  - Maps path → hash
  - HashMap Is like a dictionary: look up a word (path) to get its definition (hash)
- `current_set` = Extract just the paths from current files into a HashSet
  - Use only paths (ignore the hashes for now)
- `_db_set` = Same for database files (used later)

#### Lines 50-68: Check Current Files Against Database
```rust
for (path, new_hash) in current_files {
    if let Some(old_hash) = db_map.get(path) {
        if new_hash == old_hash {
            unchanged_count += 1;
        } else {
            modified_files.push(FileChange {
                path: path.clone(),
                status: FileStatus::Modified,
                old_hash: Some(old_hash.clone()),
                new_hash: Some(new_hash.clone()),
            });
        }
    } else {
        new_files.push(FileChange {
            path: path.clone(),
            status: FileStatus::New,
            old_hash: None,
            new_hash: Some(new_hash.clone()),
        });
    }
}
```

**Explanation**:
- `for (path, new_hash) in current_files` = Loop through each file we just scanned
- `if let Some(old_hash) = db_map.get(path)` = Look up this path in the database
  - If found, get its old hash
  - `Some(old_hash)` = Found it
- `if new_hash == old_hash` = Compare hashes
  - Same hash = unchanged, increment counter
- `else` = Different hash = modified, add to modified_files list
- `else` (of the if let) = Path not in database = new file, add to new_files list

#### Lines 71-79: Check for Deleted Files
```rust
for (path, hash) in database_files {
    if !current_set.contains(path) {
        deleted_files.push(FileChange {
            path: path.clone(),
            status: FileStatus::Deleted,
            old_hash: Some(hash.clone()),
            new_hash: None,
        });
    }
}
```

**Explanation**:
- `for (path, hash) in database_files` = Loop through database files
- `if !current_set.contains(path)` = If this path is NOT in the current scan
  - `!` means "not"
- Then it's deleted: add to deleted_files list

#### Lines 81-87: Return Results
```rust
ComparisonResult {
    new_files,
    modified_files,
    deleted_files,
    unchanged_count,
}
```

**Explanation**:
- Create and return a ComparisonResult with all our findings

#### Lines 89-165: Tests
Tests verify the comparison logic:
- New file detection: adding a file creates a "new" entry
- Modified detection: changing a hash creates a "modified" entry
- Deleted detection: missing a file creates a "deleted" entry
- Unchanged: same hash = no change

---

### File 6: `src/main.rs` - The Controller (Orchestrator)

**Purpose**: This is the main program. It coordinates all the other modules and does the actual work.

#### Lines 1-4: Import Modules
```rust
mod scanner;
mod hasher;
mod db;
mod comparator;
```

**Explanation**:
- `mod` = "module" (a separate file of code)
- These lines load the four modules we created

#### Lines 6-10: Import Tools
```rust
use clap::Parser;
use dotenvy::dotenv;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;
```

**Explanation**:
- `clap::Parser` = Tool for parsing command-line arguments
- `dotenvy::dotenv` = Tool for loading .env file
- `PathBuf` = File path type
- `OpenOptions` = Tool for opening files for writing
- `Write` = Ability to write to files

#### Lines 12-15: Import Functions from Our Modules
```rust
use scanner::scan_directory;
use hasher::compute_file_hash;
use db::Database;
use comparator::compare_hashes;
```

**Explanation**:
- `use scanner::scan_directory` = Import the scan_directory function
- Same for the other functions and types
- This lets us use them like `scan_directory(...)` instead of `scanner::scan_directory(...)`

#### Lines 17-20: Define Command-Line Arguments
```rust
#[derive(Parser, Debug)]
#[command(name = "FIM")]
#[command(about = "File Integrity Monitor - Detect file changes using SHA256 hashing", long_about = None)]
struct Args {
```

**Explanation**:
- `#[derive(Parser, Debug)]` = Make this structure parse command-line arguments
- `#[command(name = "FIM")]` = Program name is "FIM"
- `#[command(about = "...")]` = Description shown by --help

#### Lines 21-27: Argument Definitions
```rust
struct Args {
    /// Directory to scan
    #[arg(value_name = "PATH")]
    directory: PathBuf,

    /// Optional log file path
    #[arg(short, long)]
    log: Option<PathBuf>,
}
```

**Explanation**:
- `directory: PathBuf` = Required argument: the directory to scan
  - User types: `fim /home/user`
- `log: Option<PathBuf>` = Optional argument: log file path
  - User types: `fim /home/user --log /tmp/fim.log`
  - `Option` = Might be None (not provided) or Some(path)
  - `#[arg(short, long)]` = Can use `-l` or `--log`

#### Lines 29: Function Declaration
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
```

**Explanation**:
- `fn main()` = The main entry point (program starts here)
- `-> Result<...Error>` = Returns success or an error
- This lets us use `?` operator to stop on errors

#### Lines 31-36: Initialize Logging
```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("info".parse()?),
    )
    .init();
```

**Explanation**:
- `tracing_subscriber::fmt()` = Set up logging to print to console
- `.with_env_filter(...)` = Only show "info" level messages (or more important)
- `.init()` = Turn on logging

#### Lines 38-41: Load Environment Variables
```rust
dotenv().ok();

let args = Args::parse();
```

**Explanation**:
- `dotenv().ok();` = Load .env file if it exists
  - `.ok()` = Ignore if file doesn't exist (don't error)
- `Args::parse();` = Parse command-line arguments
  - If user typed: `fim /etc --log /tmp/fim.log`
  - Then args.directory = "/etc" and args.log = Some("/tmp/fim.log")

#### Lines 43-52: Validate Directory
```rust
if !args.directory.exists() {
    eprintln!("[ERROR] Directory does not exist: {}", args.directory.display());
    std::process::exit(1);
}

if !args.directory.is_dir() {
    eprintln!("[ERROR] Path is not a directory: {}", args.directory.display());
    std::process::exit(1);
}
```

**Explanation**:
- `if !args.directory.exists()` = If directory doesn't exist
- `eprintln!(...)` = Print error message to screen
- `std::process::exit(1);` = Exit program with error code 1 (means failure)
- Same check for "is it actually a directory?"

#### Lines 54-55: Print Starting Message
```rust
println!("[INFO] Starting File Integrity Monitor");
println!("[INFO] Scanning directory: {}", args.directory.display());
```

**Explanation**:
- `println!(...)` = Print to console
- `{}` = Placeholder for a value
- `.display()` = Convert path to human-readable text

#### Lines 57-65: Get Database Credentials
```rust
let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
let db_user = std::env::var("DB_USER")?;
let db_pass = std::env::var("DB_PASS")?;
let db_name = std::env::var("DB_NAME")?;
```

**Explanation**:
- `std::env::var("DB_HOST")` = Get environment variable "DB_HOST"
- `.unwrap_or_else(...) = If not found, use "localhost" as default
- For the others, `?` = If not found, stop and error out
  - User must provide these in .env file

#### Lines 67-76: Connect to Database
```rust
let db = match Database::new(&db_host, &db_user, &db_pass, &db_name) {
    Ok(d) => {
        println!("[INFO] Connected to database");
        d
    }
    Err(e) => {
        eprintln!("[ERROR] Failed to connect to database: {}", e);
        std::process::exit(1);
    }
};
```

**Explanation**:
- `Database::new(...)` = Try to create database connection
- `match` = "Check if success or failure"
  - `Ok(d)` = Success case: print message and use the database
  - `Err(e)` = Failure case: print error and exit

#### Lines 78-90: Scan Directory
```rust
let scanned_files = match scan_directory(&args.directory) {
    Ok(files) => {
        println!("[INFO] Found {} files", files.len());
        files
    }
    Err(e) => {
        eprintln!("[ERROR] Failed to scan directory: {}", e);
        std::process::exit(1);
    }
};
```

**Explanation**:
- `scan_directory(&args.directory)` = Call the scanner module
- Same match pattern: success or error

#### Lines 92-110: Hash All Files
```rust
let mut current_files = Vec::new();
let mut hash_errors = 0;

println!("[INFO] Computing hashes...");
for scanned_file in scanned_files {
    match compute_file_hash(&scanned_file.path) {
        Ok(hash) => {
            let path_str = scanned_file.path.to_string_lossy().to_string();
            current_files.push((path_str, hash));
        }
        Err(e) => {
            eprintln!(
                "[WARN] Failed to hash file {}: {}",
                scanned_file.path.display(),
                e
            );
            hash_errors += 1;
        }
    }
}
```

**Explanation**:
- Create empty list for (path, hash) pairs
- Create counter for errors
- Loop through each scanned file
- Try to hash it
  - Success: add (path, hash) to list
  - Error: print warning and increment error counter
- `to_string_lossy()` = Convert path to string (handling special characters)

#### Lines 112-114: Report Hashing Errors
```rust
if hash_errors > 0 {
    println!("[WARN] {} files failed to hash", hash_errors);
}
```

**Explanation**:
- If we had hashing errors, print how many

#### Lines 116-128: Get Database Records
```rust
let db_files = match db.get_all_files() {
    Ok(files) => files
        .into_iter()
        .map(|f| (f.path, f.hash))
        .collect::<Vec<_>>(),
    Err(e) => {
        eprintln!("[ERROR] Failed to fetch database records: {}", e);
        std::process::exit(1);
    }
};
```

**Explanation**:
- Call db.get_all_files() to get all stored records
- `.into_iter()` = Convert to loop
- `.map(|f| (f.path, f.hash))` = Extract just path and hash pairs
- `.collect::<Vec<_>>()` = Convert back to a vector
- Convert FileRecord objects to (path, hash) pairs to match our current_files format

#### Lines 130-131: Compare Hashes
```rust
let comparison = compare_hashes(&current_files, &db_files);
```

**Explanation**:
- Call the comparator module
- Get back a ComparisonResult with new, modified, deleted, unchanged

#### Lines 133-134: Prepare Log Messages
```rust
let mut log_messages = Vec::new();
```

**Explanation**:
- Create empty list to collect messages for the log file

#### Lines 136-149: Report and Process New Files
```rust
for change in &comparison.new_files {
    let msg = format!("[NEW] {}", change.path);
    println!("{}", msg);
    log_messages.push(msg);

    if let Err(e) = db.insert_file(&change.path, change.new_hash.as_ref().unwrap()) {
        eprintln!("[ERROR] Failed to insert file into database: {}", e);
    }
}
```

**Explanation**:
- `for change in &comparison.new_files` = Loop through new files
- `let msg = format!("[NEW] {}", change.path);` = Create message
- `println!("{}", msg);` = Print to console
- `log_messages.push(msg);` = Save for log file
- `db.insert_file(...)` = Add this file to database
  - `change.new_hash.as_ref().unwrap()` = Get the hash (it exists for new files)

#### Lines 151-173: Report and Process Modified Files
```rust
for change in &comparison.modified_files {
    let msg = format!(
        "[MODIFIED] {} (old: {}, new: {})",
        change.path,
        change.old_hash.as_ref().unwrap(),
        change.new_hash.as_ref().unwrap()
    );
    println!("{}", msg);
    log_messages.push(msg);

    if let Err(e) = db.update_file(&change.path, change.new_hash.as_ref().unwrap()) {
        eprintln!("[ERROR] Failed to update file in database: {}", e);
    }
}
```

**Explanation**:
- Similar to new files, but:
- Message includes old and new hashes for clarity
- Call db.update_file() instead of insert

#### Lines 175-187: Report and Process Deleted Files
```rust
for change in &comparison.deleted_files {
    let msg = format!("[DELETED] {}", change.path);
    println!("{}", msg);
    log_messages.push(msg);

    if let Err(e) = db.delete_file(&change.path) {
        eprintln!("[ERROR] Failed to delete file from database: {}", e);
    }
}
```

**Explanation**:
- Loop through deleted files
- Print message
- Call db.delete_file() to remove from database

#### Lines 189-196: Print Summary
```rust
println!(
    "\n[SUMMARY] Unchanged: {}, New: {}, Modified: {}, Deleted: {}",
    comparison.unchanged_count,
    comparison.new_files.len(),
    comparison.modified_files.len(),
    comparison.deleted_files.len()
);
```

**Explanation**:
- Print summary statistics
- `\n` = Newline (blank line before summary)
- `.len()` = Length of each list

#### Lines 198-215: Write Log File (if requested)
```rust
if let Some(log_path) = args.log {
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
```

**Explanation**:
- `if let Some(log_path) = args.log` = If user provided --log argument
- `OpenOptions::new()` = Create file opening options
- `.create(true)` = Create file if it doesn't exist
- `.append(true)` = Append to file (don't overwrite)
- `.open(&log_path)` = Open the file

```rust
        Ok(mut file) => {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(file, "\n--- {} ---", timestamp)?;
            for msg in log_messages {
                writeln!(file, "{}", msg)?;
            }
            println!("[INFO] Log written to: {}", log_path.display());
        }
```

**Explanation**:
- Success case: file opened
- `chrono::Utc::now()` = Get current time
- `.format(...)` = Format as "YYYY-MM-DD HH:MM:SS"
- `writeln!(file, ...)` = Write a line to the file
- Write timestamp header
- Write each log message
- Print confirmation message

```rust
        Err(e) => {
            eprintln!("[ERROR] Failed to write log file: {}", e);
        }
    }
}

Ok(())
```

**Explanation**:
- Error case: couldn't open file, print error
- `Ok(())` = Return success from main function
- Empty tuple `()` = No special return value

---

## Configuration and Deployment Files

### File 7: `.env.example` - Environment Configuration Template

This file serves as a template for environment variables. It demonstrates which configuration parameters are required before running the application.

```bash
# Database Configuration
DB_HOST=localhost
DB_USER=fim_user
DB_PASS=your_secure_password
DB_NAME=file_integrity_monitor

# Logging
RUST_LOG=info
```

**Explanation of Variables:**

- **DB_HOST=localhost**: The hostname or IP address where the MySQL/MariaDB server is running. `localhost` refers to the local machine. For remote databases, use a fully qualified domain name or IP address.

- **DB_USER=fim_user**: The database user account that has permission to access the database. This user should be created in MySQL with appropriate privileges (CREATE, INSERT, UPDATE, DELETE, SELECT) on the specified database.

- **DB_PASS=your_secure_password**: The password for the database user. This should be a strong, random password. In production, use a secrets management system rather than plain-text environment variables.

- **DB_NAME=file_integrity_monitor**: The MySQL database name where file integrity records will be stored. This database should exist before running the application (created manually via MySQL).

- **RUST_LOG=info**: Controls the logging verbosity level for the `tracing` infrastructure. Acceptable values are: `error`, `warn`, `info`, `debug`, `trace`. `info` is recommended for production; use `debug` for troubleshooting.

**Usage**: Copy this file to `.env` in the project root directory, then edit with actual database credentials:
```bash
cp .env.example .env
nano .env  # Edit with your credentials
```

---

### File 8: `script.sh` - Shell Script Execution Wrapper

This is a Bash shell script that provides a convenient entry point for running the FIM application. It handles environment variable loading and build verification.

```bash
#!/bin/bash
```
**Explanation**: The shebang line (`#!`) specifies that this file should be executed with the `/bin/bash` interpreter.

```bash
set -e
```
**Explanation**: The `set -e` option causes the script to exit immediately if any command exits with a non-zero status (error). This prevents subsequent commands from executing after a failure.

```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
```
**Explanation**: This line computes the absolute path of the directory containing the script itself. 
- `"${BASH_SOURCE[0]}"` refers to the script's filename
- `dirname` extracts the directory path
- `cd` changes to that directory
- `pwd` prints the working directory (now the script's directory)
- The entire expression is wrapped in `$(...)` for command substitution

```bash
if [ -f "$SCRIPT_DIR/.env" ]; then
    set -a
    source "$SCRIPT_DIR/.env"
    set +a
else
    echo "[ERROR] .env file not found in $SCRIPT_DIR"
    exit 1
fi
```
**Explanation**: This block checks if `.env` exists and loads its variables:
- `[ -f "$SCRIPT_DIR/.env" ]` tests if the file exists and is a regular file
- `set -a` enables automatic export of all variable assignments (makes them environment variables)
- `source` executes the `.env` file, loading all variable definitions into the current shell environment
- `set +a` disables automatic export
- `else` block exits with error code 1 if the file is missing

```bash
BINARY="$SCRIPT_DIR/target/release/fim"
if [ ! -f "$BINARY" ]; then
    echo "[ERROR] Binary not found at $BINARY"
    echo "[INFO] Building FIM..."
    cd "$SCRIPT_DIR"
    cargo build --release
fi
```
**Explanation**: Checks if the compiled binary exists; if not, compiles it:
- `[ ! -f "$BINARY" ]` tests if the file does NOT exist (`!` is logical negation)
- If missing, prints diagnostic info and invokes `cargo build --release` to compile

```bash
SCAN_DIR="${1:-.}"
```
**Explanation**: Uses the first command-line argument as the directory to scan, defaulting to `.` (current directory) if not provided:
- `${1:-...}` is parameter expansion with a default value
- `${1}` is the first positional argument
- `:-` specifies that if `$1` is unset or empty, use the following default
- `.` is the current directory

```bash
LOG_FILE="/var/log/fim.log"
if [ ! -f "$LOG_FILE" ] 2>/dev/null; then
    LOG_FILE="/tmp/fim-$(date +%s).log"
fi
```
**Explanation**: Attempts to use `/var/log/fim.log`, falling back to a timestamped file in `/tmp`:
- Tries to access `/var/log/fim.log`
- `2>/dev/null` redirects error messages to null (suppresses them)
- If the file doesn't exist or isn't accessible, creates an alternative path with a Unix timestamp (`$(date +%s)`)

```bash
echo "[INFO] Starting FIM scan at $(date)"
"$BINARY" "$SCAN_DIR" --log "$LOG_FILE"
echo "[INFO] FIM scan completed at $(date)"
```
**Explanation**: Executes the FIM binary with appropriate arguments:
- First echo announces the start time
- Invokes the compiled binary with the scan directory and log file path
- Final echo marks completion

---

### File 9: `fim.service` - Systemd Service Unit

This file defines a systemd service unit that allows FIM to be managed by the systemd init system on Linux.

```ini
[Unit]
Description=File Integrity Monitor Service
Documentation=file:///opt/fim/README.md
After=network.target mysql.service mariadb.service
```
**Explanation**: The `[Unit]` section defines metadata and dependencies:
- `Description`: Human-readable service name displayed in systemctl output
- `Documentation`: Path to documentation (here, a local readme file)
- `After=network.target mysql.service mariadb.service`: Specifies ordering constraints. This service should start AFTER these other units. `network.target` is a system target guaranteed after network is ready; `mysql.service` and `mariadb.service` ensure the database is running before FIM attempts connection.

```ini
[Service]
Type=oneshot
User=root
Group=root
WorkingDirectory=/opt/fim
ExecStart=/opt/fim/script.sh /etc
StandardOutput=journal
StandardError=journal
PrivateTmp=no
```
**Explanation**: The `[Service]` section defines how the service executes:
- `Type=oneshot`: Indicates the service runs once and exits (as opposed to `simple` for long-running daemons). Systemd waits for the process to complete.
- `User=root` and `Group=root`: The service runs with root privileges (necessary for scanning system directories like `/etc`)
- `WorkingDirectory=/opt/fim`: Sets the current working directory when the process starts
- `ExecStart=/opt/fim/script.sh /etc`: The command to execute. Here, runs the wrapper script with `/etc` as the scan target.
- `StandardOutput=journal` and `StandardError=journal`: Directs all output to the systemd journal (journalctl log system)
- `PrivateTmp=no`: The service uses the system's `/tmp` rather than a private temporary directory

```ini
[Install]
WantedBy=multi-user.target
```
**Explanation**: The `[Install]` section defines how the service integrates with systemd targets:
- `WantedBy=multi-user.target`: Specifies that this service is wanted by the multi-user target (the normal Linux boot state). When enabled, a symlink is created so the service starts on boot.

---

### File 10: `fim.timer` - Systemd Timer Unit

This file defines a systemd timer that schedules the FIM service to run periodically, replacing traditional cron jobs.

```ini
[Unit]
Description=File Integrity Monitor Daily Scan Timer
Documentation=file:///opt/fim/README.md
Requires=fim.service
```
**Explanation**: The `[Unit]` section defines timer metadata:
- `Description`: Describes the timer's purpose
- `Documentation`: Links to documentation
- `Requires=fim.service`: Declares a hard dependency on the `fim.service` unit. The timer cannot function without the service being defined.

```ini
[Timer]
OnBootSec=5min
OnUnitActiveSec=1d
OnCalendar=*-*-* 02:00:00
Persistent=true
```
**Explanation**: The `[Timer]` section defines scheduling behavior:
- `OnBootSec=5min`: Executes 5 minutes after the system boots. Useful for running checks early in the startup process.
- `OnUnitActiveSec=1d`: Executes again 1 day (24 hours) after the previous execution completed. `d` is day unit; other units: `s` (seconds), `m` (minutes), `h` (hours).
- `OnCalendar=*-*-* 02:00:00`: A calendar-based schedule using systemd's calendar syntax. `*-*-*` means any day, `02:00:00` means 2:00 AM UTC. This ensures a consistent daily schedule independent of reboot state.
- `Persistent=true`: If the timer is stopped and then restarted, and the scheduled time has passed, the timer triggers immediately upon restart. Prevents missed executions after system downtime.

```ini
[Install]
WantedBy=timers.target
```
**Explanation**: The `[Install]` section defines integration with systemd targets:
- `WantedBy=timers.target`: Specifies that this timer is wanted by the `timers.target` (the aggregated target for all timers). When enabled, systemd creates a symlink so the timer starts when the `timers.target` is reached on boot.

**Systemd Calendar Expression Syntax:**
The `OnCalendar=` directive supports a flexible expression format:
- `*-*-* 02:00:00` = Every day at 2:00 AM
- `Mon *-*-* 00:00:00` = Every Monday at midnight
- `*-*-01 12:00:00` = First day of each month at noon
- `*-01,07-* 18:30:00` = 18:30 on the 1st and 7th of every month

---

## Module Interaction Model

Here's how the modules interact:

```
┌─────────────────────────────────────────────────────────────┐
│                          main.rs                            │
│  (Orchestrates everything, calls other modules in order)   │
└──────────┬──────────────┬──────────────┬────────────────────┘
           │              │              │
           ▼              ▼              ▼
    ┌─────────┐    ┌────────────┐   ┌──────────────┐
    │scanner  │    │  hasher    │   │  database    │
    │─────────│    │────────────│   │──────────────│
    │ Finds   │    │ Computes   │   │ Stores and   │
    │ files   │    │ SHA256     │   │ retrieves    │
    │ in dir  │    │ hashes     │   │ file records │
    └────┬────┘    └────┬───────┘   └───────┬──────┘
         │              │                   │
         └──────────────┼───────────────────┘
                        │
                        ▼
                  ┌──────────────┐
                  │ comparator   │
                  │──────────────│
                  │ Detects and  │
                  │ categorizes  │
                  │ file changes │
                  └──────────────┘
```

### The Flow:

1. **main.rs starts**
   - Loads .env file with database credentials
   - Parses command-line arguments
   - Validates the directory exists

2. **Calls scanner.rs**
   - Walks through all folders and subfolders
   - Returns a list of all files found

3. **Calls hasher.rs for each file**
   - Reads each file's contents in 8KB chunks
   - Computes SHA256 hash
   - Returns list of (path, hash) pairs

4. **Calls database.rs to get old records**
   - Connects to MySQL/MariaDB database
   - Retrieves all previously stored (path, hash) pairs

5. **Calls comparator.rs to find changes**
   - Compares new hashes with stored hashes
   - Categorizes files as: NEW, MODIFIED, UNCHANGED, DELETED

6. **Updates database through database.rs**
   - Inserts new files
   - Updates modified files with new hashes
   - Deletes removed files

7. **Reports results**
   - Prints to console
   - Optionally writes to log file

---

## Complete Workflow

Here's what happens when you run: `fim /etc --log /tmp/fim.log`

### Step 1: Startup
```
1. Load .env file (get DB_HOST, DB_USER, DB_PASS, DB_NAME)
2. Parse arguments:
   - directory = /etc
   - log = /tmp/fim.log
3. Check /etc exists and is a directory
4. Print: "[INFO] Starting File Integrity Monitor"
5. Print: "[INFO] Scanning directory: /etc"
```

### Step 2: Connect to Database
```
6. Load DB_HOST (or use "localhost" as default)
7. Load DB_USER, DB_PASS, DB_NAME from .env
8. Connect to MySQL/MariaDB
9. Create "files" table if it doesn't exist
10. Print: "[INFO] Connected to database"
```

### Step 3: Scan Directory
```
11. Walk through /etc recursively
12. Find all files (skip directories)
13. Build list of all file paths
14. Print: "[INFO] Found 1234 files"
```

### Step 4: Compute Hashes
```
15. For each file found:
    a. Open file
    b. Read in 8KB chunks
    c. Feed to SHA256 hasher
    d. Get 64-character hex hash
    e. Store (path, hash) pair
    f. If error: log warning, continue
16. Print: "[INFO] Computing hashes..."
17. If any errors: Print: "[WARN] X files failed to hash"
```

### Step 5: Fetch Old Records
```
18. Query database: SELECT * FROM files
19. Convert to (path, hash) pairs
20. Build HashMap for fast lookups
```

### Step 6: Compare
```
21. For each current file:
    a. Look up in database
    b. If found:
       - Compare hashes
       - Same: unchanged
       - Different: modified
    c. If not found: new
22. For each database file:
    a. Check if still exists on disk
    b. If not: deleted
```

### Step 7: Report and Update
```
For each NEW file:
23. Print: "[NEW] /etc/somefile"
24. Add to log_messages
25. INSERT INTO database

For each MODIFIED file:
26. Print: "[MODIFIED] /etc/somefile (old: abc..., new: def...)"
27. Add to log_messages
28. UPDATE database

For each DELETED file:
29. Print: "[DELETED] /etc/oldfile"
30. Add to log_messages
31. DELETE from database

32. Print: "[SUMMARY] Unchanged: 1200, New: 2, Modified: 1, Deleted: 1"
```

### Step 8: Write Log File
```
33. Open /tmp/fim.log for appending
34. Write timestamp: "--- 2024-04-08 14:30:45 ---"
35. Write each log message
36. Close file
37. Print: "[INFO] Log written to: /tmp/fim.log"
```

### Step 9: Exit
```
38. Return Ok(()) (success)
39. Program ends
```

---

## Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                      File System                            │
│  /etc/
│  ├── file1.txt (contents...)
│  ├── file2.txt (contents...)
│  └── file3.txt (contents...)
└──────────────┬───────────────────────────────────────────────┘
               │
               ▼ scanner.rs
        ┌─────────────────────┐
        │ Scanned Files List  │
        ├─────────────────────┤
        │ /etc/file1.txt      │
        │ /etc/file2.txt      │
        │ /etc/file3.txt      │
        └────┬────────────────┘
             │
             ▼ hasher.rs (for each file)
        ┌──────────────────────────────┐
        │ Current (path, hash) pairs   │
        ├──────────────────────────────┤
        │ ("/etc/file1.txt", "abc123") │
        │ ("/etc/file2.txt", "def456") │
        │ ("/etc/file3.txt", "ghi789") │
        └────────┬─────────────────────┘
                 │
    ┌────────────┴────────────┐
    │                         │
    ▼              ▼ db.rs
    │    ┌──────────────────────────┐
    │    │ Stored (path, hash)      │
    │    ├──────────────────────────┤
    │    │ ("/etc/file1.txt","abc123") │
    │    │ ("/etc/file2.txt","old456") │  ◄── file2 changed!
    │    │ ("/etc/file4.txt","jkl000") │  ◄── file4 deleted!
    │    └──────────────────────────┘
    │
    └────────────┬────────────┘
                 │
                 ▼ comparator.rs
        ┌────────────────────────────┐
        │ Comparison Results         │
        ├────────────────────────────┤
        │ NEW: /etc/file3.txt        │
        │ MODIFIED: /etc/file2.txt   │
        │ DELETED: /etc/file4.txt    │
        │ UNCHANGED: 1 (file1)       │
        └────────┬───────────────────┘
                 │
    ┌────────────┴────────────┐
    │                         │
    ▼              ▼ db.rs (update)
    │    ┌──────────────────────────┐
    │    │ Updated Database         │
    │    ├──────────────────────────┤
    │    │ ("/etc/file1.txt","abc123") │
    │    │ ("/etc/file2.txt","def456") │  ◄── updated
    │    │ ("/etc/file3.txt","ghi789") │  ◄── inserted
    │    └──────────────────────────┘
    │
    └──► Console Output
    
         [NEW] /etc/file3.txt
         [MODIFIED] /etc/file2.txt (old: old456, new: def456)
         [DELETED] /etc/file4.txt
         [SUMMARY] Unchanged: 1, New: 1, Modified: 1, Deleted: 1
```

---

## Summary

**In Plain English:**

1. **scanner.rs** is like a robot that explores a building room by room and makes a list of everything it finds.

2. **hasher.rs** is like a fingerprint scanner that takes each item from the robot's list and creates a unique fingerprint for it.

3. **db.rs** is like a record keeper that has a filing cabinet (database) where it stores these fingerprints, and can add, update, or remove records.

4. **comparator.rs** is like a detective that compares today's fingerprints with yesterday's fingerprints to see what's new, what changed, what was deleted.

5. **main.rs** is the boss that tells everyone else what to do, in order, and reports the results.

Together, they create a system that watches for file tampering by comparing file fingerprints over time.
