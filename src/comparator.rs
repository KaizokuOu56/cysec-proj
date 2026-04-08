use std::collections::HashSet;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FileStatus {
    New,
    Modified,
    Unchanged,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub status: FileStatus,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
}

/// Result of a complete file integrity check
#[derive(Debug)]
pub struct ComparisonResult {
    pub new_files: Vec<FileChange>,
    pub modified_files: Vec<FileChange>,
    pub deleted_files: Vec<FileChange>,
    pub unchanged_count: usize,
}

impl ComparisonResult {
    pub fn total_changes(&self) -> usize {
        self.new_files.len() + self.modified_files.len() + self.deleted_files.len()
    }
}

/// Compares current file state with database records
pub fn compare_hashes(
    current_files: &[(String, String)], // (path, hash)
    database_files: &[(String, String)], // (path, hash)
) -> ComparisonResult {
    let mut new_files = Vec::new();
    let mut modified_files = Vec::new();
    let mut deleted_files = Vec::new();
    let mut unchanged_count = 0;

    // Convert database to hashmap for O(1) lookups
    let db_map: std::collections::HashMap<_, _> = database_files.iter().cloned().collect();
    let current_set: HashSet<_> = current_files.iter().map(|(p, _)| p.clone()).collect();
    let _db_set: HashSet<_> = database_files.iter().map(|(p, _)| p.clone()).collect();

    // Check current files against database
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

    // Check for deleted files (in DB but not in current scan)
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

    ComparisonResult {
        new_files,
        modified_files,
        deleted_files,
        unchanged_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_file_detection() {
        let current = vec![
            ("file1.txt".to_string(), "hash1".to_string()),
            ("file2.txt".to_string(), "hash2".to_string()),
        ];
        let database = vec![("file1.txt".to_string(), "hash1".to_string())];

        let result = compare_hashes(&current, &database);

        assert_eq!(result.new_files.len(), 1);
        assert_eq!(result.new_files[0].path, "file2.txt");
        assert_eq!(result.modified_files.len(), 0);
        assert_eq!(result.deleted_files.len(), 0);
    }

    #[test]
    fn test_modified_file_detection() {
        let current = vec![("file1.txt".to_string(), "newhash".to_string())];
        let database = vec![("file1.txt".to_string(), "oldhash".to_string())];

        let result = compare_hashes(&current, &database);

        assert_eq!(result.modified_files.len(), 1);
        assert_eq!(result.modified_files[0].path, "file1.txt");
        assert_eq!(result.new_files.len(), 0);
        assert_eq!(result.deleted_files.len(), 0);
    }

    #[test]
    fn test_deleted_file_detection() {
        let current = vec![];
        let database = vec![("file1.txt".to_string(), "hash1".to_string())];

        let result = compare_hashes(&current, &database);

        assert_eq!(result.deleted_files.len(), 1);
        assert_eq!(result.deleted_files[0].path, "file1.txt");
        assert_eq!(result.new_files.len(), 0);
        assert_eq!(result.modified_files.len(), 0);
    }

    #[test]
    fn test_unchanged_file() {
        let current = vec![("file1.txt".to_string(), "hash1".to_string())];
        let database = vec![("file1.txt".to_string(), "hash1".to_string())];

        let result = compare_hashes(&current, &database);

        assert_eq!(result.unchanged_count, 1);
        assert_eq!(result.new_files.len(), 0);
        assert_eq!(result.modified_files.len(), 0);
        assert_eq!(result.deleted_files.len(), 0);
    }
}
