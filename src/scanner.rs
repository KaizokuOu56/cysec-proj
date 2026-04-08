use std::path::{Path, PathBuf};
use walkdir::{WalkDir, DirEntry};

/// Represents a file found during scanning
#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub path: PathBuf,
}

/// Recursively scans a directory and yields all files
/// Skips directories that cannot be read and continues gracefully
pub fn scan_directory(root: &Path) -> Result<Vec<ScannedFile>, Box<dyn std::error::Error>> {
    if !root.exists() {
        return Err(format!("Directory does not exist: {}", root.display()).into());
    }

    if !root.is_dir() {
        return Err(format!("Path is not a directory: {}", root.display()).into());
    }

    let mut files = Vec::new();

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
    {
        // Skip directories, only process files
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let path = entry.path().to_path_buf();
                    files.push(ScannedFile { path });
                }
            }
        }
    }

    Ok(files)
}

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

    #[test]
    fn test_scan_directory_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let _file1 = File::create(temp_dir.path().join("file1.txt")).unwrap();
        let _file2 = File::create(temp_dir.path().join("file2.txt")).unwrap();

        let files = scan_directory(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let result = scan_directory(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
