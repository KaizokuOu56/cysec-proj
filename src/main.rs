mod scanner;
mod hasher;
mod db;
mod comparator;

use clap::Parser;
use dotenvy::dotenv;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;

use scanner::scan_directory;
use hasher::compute_file_hash;
use db::Database;
use comparator::compare_hashes;

#[derive(Parser, Debug)]
#[command(name = "FIM")]
#[command(about = "File Integrity Monitor - Detect file changes using SHA256 hashing", long_about = None)]
struct Args {
    /// Directory to scan
    #[arg(value_name = "PATH")]
    directory: PathBuf,

    /// Optional log file path
    #[arg(short, long)]
    log: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse()?),
        )
        .init();

    // Load .env file
    dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Validate directory path
    if !args.directory.exists() {
        eprintln!("[ERROR] Directory does not exist: {}", args.directory.display());
        std::process::exit(1);
    }

    if !args.directory.is_dir() {
        eprintln!("[ERROR] Path is not a directory: {}", args.directory.display());
        std::process::exit(1);
    }

    println!("[INFO] Starting File Integrity Monitor");
    println!("[INFO] Scanning directory: {}", args.directory.display());

    // Connect to database
    let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_user = std::env::var("DB_USER")?;
    let db_pass = std::env::var("DB_PASS")?;
    let db_name = std::env::var("DB_NAME")?;

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

    // Scan directory for files
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

    // Compute hashes for all files
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

    if hash_errors > 0 {
        println!("[WARN] {} files failed to hash", hash_errors);
    }

    // Fetch existing records from database
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

    // Compare hashes
    let comparison = compare_hashes(&current_files, &db_files);

    // Log file changes
    let mut log_messages = Vec::new();

    // Report new files
    for change in &comparison.new_files {
        let msg = format!("[NEW] {}", change.path);
        println!("{}", msg);
        log_messages.push(msg);

        if let Err(e) = db.insert_file(&change.path, change.new_hash.as_ref().unwrap()) {
            eprintln!("[ERROR] Failed to insert file into database: {}", e);
        }
    }

    // Report modified files
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

    // Report deleted files
    for change in &comparison.deleted_files {
        let msg = format!("[DELETED] {}", change.path);
        println!("{}", msg);
        log_messages.push(msg);

        if let Err(e) = db.delete_file(&change.path) {
            eprintln!("[ERROR] Failed to delete file from database: {}", e);
        }
    }

    // Print summary
    println!(
        "\n[SUMMARY] Unchanged: {}, New: {}, Modified: {}, Deleted: {}",
        comparison.unchanged_count,
        comparison.new_files.len(),
        comparison.modified_files.len(),
        comparison.deleted_files.len()
    );

    // Write to log file if specified
    if let Some(log_path) = args.log {
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            Ok(mut file) => {
                let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
                writeln!(file, "\n--- {} ---", timestamp)?;
                for msg in log_messages {
                    writeln!(file, "{}", msg)?;
                }
                println!("[INFO] Log written to: {}", log_path.display());
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to write log file: {}", e);
            }
        }
    }

    Ok(())
}
