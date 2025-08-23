//! File system utilities for the LedgerDB blockchain.
//!
//! This module provides file system operations, directory management,
//! and file handling utilities.

use crate::error::LedgerError;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// File system utilities
pub struct FileSystemUtils;

impl FileSystemUtils {
    /// Create directory if it doesn't exist
    pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> Result<(), LedgerError> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| {
                LedgerError::Io(format!("Failed to create directory '{}': {}", path.display(), e))
            })?;
        }
        Ok(())
    }
    
    /// Check if file exists
    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_file()
    }
    
    /// Check if directory exists
    pub fn dir_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_dir()
    }
    
    /// Get file size
    pub fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64, LedgerError> {
        let metadata = fs::metadata(path.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to get metadata for '{}': {}",
                path.as_ref().display(),
                e
            ))
        })?;
        Ok(metadata.len())
    }
    
    /// Get file modification time
    pub fn get_file_modified_time<P: AsRef<Path>>(path: P) -> Result<SystemTime, LedgerError> {
        let metadata = fs::metadata(path.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to get metadata for '{}': {}",
                path.as_ref().display(),
                e
            ))
        })?;
        metadata.modified().map_err(|e| {
            LedgerError::Io(format!(
                "Failed to get modification time for '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }
    
    /// Read file to string
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, LedgerError> {
        fs::read_to_string(path.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to read file '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }
    
    /// Read file to bytes
    pub fn read_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, LedgerError> {
        fs::read(path.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to read file '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }
    
    /// Write string to file
    pub fn write_string<P: AsRef<Path>>(path: P, content: &str) -> Result<(), LedgerError> {
        fs::write(path.as_ref(), content).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to write to file '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }
    
    /// Write bytes to file
    pub fn write_bytes<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<(), LedgerError> {
        fs::write(path.as_ref(), content).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to write to file '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }
    
    /// Append string to file
    pub fn append_string<P: AsRef<Path>>(path: P, content: &str) -> Result<(), LedgerError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())
            .map_err(|e| {
                LedgerError::Io(format!(
                    "Failed to open file '{}' for appending: {}",
                    path.as_ref().display(),
                    e
                ))
            })?;
        
        file.write_all(content.as_bytes()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to append to file '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }
    
    /// Copy file
    pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(
        from: P,
        to: Q,
    ) -> Result<u64, LedgerError> {
        fs::copy(from.as_ref(), to.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to copy file from '{}' to '{}': {}",
                from.as_ref().display(),
                to.as_ref().display(),
                e
            ))
        })
    }
    
    /// Move/rename file
    pub fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(
        from: P,
        to: Q,
    ) -> Result<(), LedgerError> {
        fs::rename(from.as_ref(), to.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to move file from '{}' to '{}': {}",
                from.as_ref().display(),
                to.as_ref().display(),
                e
            ))
        })
    }
    
    /// Delete file
    pub fn delete_file<P: AsRef<Path>>(path: P) -> Result<(), LedgerError> {
        fs::remove_file(path.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to delete file '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }
    
    /// Delete directory (recursively)
    pub fn delete_dir<P: AsRef<Path>>(path: P) -> Result<(), LedgerError> {
        fs::remove_dir_all(path.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to delete directory '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }
    
    /// List directory contents
    pub fn list_dir<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, LedgerError> {
        let entries = fs::read_dir(path.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to read directory '{}': {}",
                path.as_ref().display(),
                e
            ))
        })?;
        
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                LedgerError::Io(format!(
                    "Failed to read directory entry in '{}': {}",
                    path.as_ref().display(),
                    e
                ))
            })?;
            paths.push(entry.path());
        }
        
        Ok(paths)
    }
    
    /// Get directory size (recursive)
    pub fn get_dir_size<P: AsRef<Path>>(path: P) -> Result<u64, LedgerError> {
        let mut total_size = 0;
        
        fn visit_dir(dir: &Path, total_size: &mut u64) -> Result<(), LedgerError> {
            let entries = fs::read_dir(dir).map_err(|e| {
                LedgerError::Io(format!("Failed to read directory '{}': {}", dir.display(), e))
            })?;
            
            for entry in entries {
                let entry = entry.map_err(|e| {
                    LedgerError::Io(format!(
                        "Failed to read directory entry in '{}': {}",
                        dir.display(),
                        e
                    ))
                })?;
                
                let path = entry.path();
                let metadata = entry.metadata().map_err(|e| {
                    LedgerError::Io(format!(
                        "Failed to get metadata for '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
                
                if metadata.is_file() {
                    *total_size += metadata.len();
                } else if metadata.is_dir() {
                    visit_dir(&path, total_size)?;
                }
            }
            
            Ok(())
        }
        
        visit_dir(path.as_ref(), &mut total_size)?;
        Ok(total_size)
    }
    
    /// Create temporary file
    pub fn create_temp_file(prefix: &str, suffix: &str) -> Result<(File, PathBuf), LedgerError> {
        use std::env;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let temp_dir = env::temp_dir();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        let filename = format!("{}{}{}", prefix, timestamp, suffix);
        let temp_path = temp_dir.join(filename);
        
        let file = File::create(&temp_path).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to create temporary file '{}': {}",
                temp_path.display(),
                e
            ))
        })?;
        
        Ok((file, temp_path))
    }
    
    /// Create backup of file
    pub fn create_backup<P: AsRef<Path>>(path: P) -> Result<PathBuf, LedgerError> {
        let path = path.as_ref();
        let backup_path = path.with_extension(
            format!(
                "{}.backup.{}",
                path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            )
        );
        
        Self::copy_file(path, &backup_path)?;
        Ok(backup_path)
    }
    
    /// Atomic write (write to temp file, then rename)
    pub fn atomic_write<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<(), LedgerError> {
        let path = path.as_ref();
        let temp_path = path.with_extension("tmp");
        
        // Write to temporary file
        Self::write_bytes(&temp_path, content)?;
        
        // Atomically rename to final path
        Self::move_file(&temp_path, path)?;
        
        Ok(())
    }
    
    /// Safe file write with backup
    pub fn safe_write<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<(), LedgerError> {
        let path = path.as_ref();
        
        // Create backup if file exists
        if path.exists() {
            Self::create_backup(path)?;
        }
        
        // Atomic write
        Self::atomic_write(path, content)
    }
}

/// File reader with buffering
pub struct BufferedFileReader {
    reader: BufReader<File>,
    path: PathBuf,
}

impl BufferedFileReader {
    /// Create new buffered file reader
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, LedgerError> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path).map_err(|e| {
            LedgerError::Io(format!("Failed to open file '{}': {}", path.display(), e))
        })?;
        
        Ok(Self {
            reader: BufReader::new(file),
            path,
        })
    }
    
    /// Read line
    pub fn read_line(&mut self) -> Result<Option<String>, LedgerError> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Ok(Some(line))
            }
            Err(e) => Err(LedgerError::Io(format!(
                "Failed to read line from '{}': {}",
                self.path.display(),
                e
            ))),
        }
    }
    
    /// Read all lines
    pub fn read_lines(&mut self) -> Result<Vec<String>, LedgerError> {
        let mut lines = Vec::new();
        while let Some(line) = self.read_line()? {
            lines.push(line);
        }
        Ok(lines)
    }
    
    /// Read bytes
    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<usize, LedgerError> {
        self.reader.read(buf).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to read bytes from '{}': {}",
                self.path.display(),
                e
            ))
        })
    }
    
    /// Get file path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// File writer with buffering
pub struct BufferedFileWriter {
    writer: BufWriter<File>,
    path: PathBuf,
}

impl BufferedFileWriter {
    /// Create new buffered file writer
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, LedgerError> {
        let path = path.as_ref().to_path_buf();
        let file = File::create(&path).map_err(|e| {
            LedgerError::Io(format!("Failed to create file '{}': {}", path.display(), e))
        })?;
        
        Ok(Self {
            writer: BufWriter::new(file),
            path,
        })
    }
    
    /// Create new buffered file writer in append mode
    pub fn new_append<P: AsRef<Path>>(path: P) -> Result<Self, LedgerError> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| {
                LedgerError::Io(format!(
                    "Failed to open file '{}' for appending: {}",
                    path.display(),
                    e
                ))
            })?;
        
        Ok(Self {
            writer: BufWriter::new(file),
            path,
        })
    }
    
    /// Write string
    pub fn write_string(&mut self, content: &str) -> Result<(), LedgerError> {
        self.writer.write_all(content.as_bytes()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to write to file '{}': {}",
                self.path.display(),
                e
            ))
        })
    }
    
    /// Write line
    pub fn write_line(&mut self, line: &str) -> Result<(), LedgerError> {
        self.write_string(line)?;
        self.write_string("\n")
    }
    
    /// Write bytes
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), LedgerError> {
        self.writer.write_all(bytes).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to write bytes to file '{}': {}",
                self.path.display(),
                e
            ))
        })
    }
    
    /// Flush buffer
    pub fn flush(&mut self) -> Result<(), LedgerError> {
        self.writer.flush().map_err(|e| {
            LedgerError::Io(format!(
                "Failed to flush file '{}': {}",
                self.path.display(),
                e
            ))
        })
    }
    
    /// Get file path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for BufferedFileWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Directory utilities
pub struct DirectoryUtils;

impl DirectoryUtils {
    /// Find files with extension
    pub fn find_files_with_extension<P: AsRef<Path>>(
        dir: P,
        extension: &str,
        recursive: bool,
    ) -> Result<Vec<PathBuf>, LedgerError> {
        let mut files = Vec::new();
        
        fn visit_dir(
            dir: &Path,
            extension: &str,
            recursive: bool,
            files: &mut Vec<PathBuf>,
        ) -> Result<(), LedgerError> {
            let entries = fs::read_dir(dir).map_err(|e| {
                LedgerError::Io(format!("Failed to read directory '{}': {}", dir.display(), e))
            })?;
            
            for entry in entries {
                let entry = entry.map_err(|e| {
                    LedgerError::Io(format!(
                        "Failed to read directory entry in '{}': {}",
                        dir.display(),
                        e
                    ))
                })?;
                
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == extension {
                            files.push(path);
                        }
                    }
                } else if path.is_dir() && recursive {
                    visit_dir(&path, extension, recursive, files)?;
                }
            }
            
            Ok(())
        }
        
        visit_dir(dir.as_ref(), extension, recursive, &mut files)?;
        Ok(files)
    }
    
    /// Clean directory (remove all contents)
    pub fn clean_directory<P: AsRef<Path>>(dir: P) -> Result<(), LedgerError> {
        let entries = fs::read_dir(dir.as_ref()).map_err(|e| {
            LedgerError::Io(format!(
                "Failed to read directory '{}': {}",
                dir.as_ref().display(),
                e
            ))
        })?;
        
        for entry in entries {
            let entry = entry.map_err(|e| {
                LedgerError::Io(format!(
                    "Failed to read directory entry in '{}': {}",
                    dir.as_ref().display(),
                    e
                ))
            })?;
            
            let path = entry.path();
            
            if path.is_file() {
                fs::remove_file(&path).map_err(|e| {
                    LedgerError::Io(format!(
                        "Failed to remove file '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
            } else if path.is_dir() {
                fs::remove_dir_all(&path).map_err(|e| {
                    LedgerError::Io(format!(
                        "Failed to remove directory '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_file_operations() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test_file.txt");
        let content = "Hello, World!";
        
        // Write file
        assert!(FileSystemUtils::write_string(&test_file, content).is_ok());
        
        // Check file exists
        assert!(FileSystemUtils::file_exists(&test_file));
        
        // Read file
        let read_content = FileSystemUtils::read_to_string(&test_file).unwrap();
        assert_eq!(read_content, content);
        
        // Get file size
        let size = FileSystemUtils::get_file_size(&test_file).unwrap();
        assert_eq!(size, content.len() as u64);
        
        // Delete file
        assert!(FileSystemUtils::delete_file(&test_file).is_ok());
        assert!(!FileSystemUtils::file_exists(&test_file));
    }
    
    #[test]
    fn test_directory_operations() {
        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_dir");
        
        // Create directory
        assert!(FileSystemUtils::ensure_dir_exists(&test_dir).is_ok());
        assert!(FileSystemUtils::dir_exists(&test_dir));
        
        // Create test file in directory
        let test_file = test_dir.join("test.txt");
        assert!(FileSystemUtils::write_string(&test_file, "test").is_ok());
        
        // List directory
        let contents = FileSystemUtils::list_dir(&test_dir).unwrap();
        assert_eq!(contents.len(), 1);
        
        // Get directory size
        let size = FileSystemUtils::get_dir_size(&test_dir).unwrap();
        assert!(size > 0);
        
        // Clean directory
        assert!(DirectoryUtils::clean_directory(&test_dir).is_ok());
        let contents = FileSystemUtils::list_dir(&test_dir).unwrap();
        assert_eq!(contents.len(), 0);
        
        // Delete directory
        assert!(FileSystemUtils::delete_dir(&test_dir).is_ok());
        assert!(!FileSystemUtils::dir_exists(&test_dir));
    }
    
    #[test]
    fn test_buffered_file_operations() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("buffered_test.txt");
        
        // Write with buffered writer
        {
            let mut writer = BufferedFileWriter::new(&test_file).unwrap();
            writer.write_line("Line 1").unwrap();
            writer.write_line("Line 2").unwrap();
            writer.flush().unwrap();
        }
        
        // Read with buffered reader
        {
            let mut reader = BufferedFileReader::new(&test_file).unwrap();
            let lines = reader.read_lines().unwrap();
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0], "Line 1");
            assert_eq!(lines[1], "Line 2");
        }
        
        // Clean up
        let _ = FileSystemUtils::delete_file(&test_file);
    }
    
    #[test]
    fn test_atomic_write() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("atomic_test.txt");
        let content = b"Atomic content";
        
        assert!(FileSystemUtils::atomic_write(&test_file, content).is_ok());
        
        let read_content = FileSystemUtils::read_to_bytes(&test_file).unwrap();
        assert_eq!(read_content, content);
        
        // Clean up
        let _ = FileSystemUtils::delete_file(&test_file);
    }
}