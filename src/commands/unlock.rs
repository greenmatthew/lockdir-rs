use std::fs::{self, remove_file};
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn unlock_directory(dir_path: Option<&str>) -> Result<()> {
    // Determine the directory path to unlock
    let path = match dir_path {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()?, // Use current directory if no path provided
    };

    // Validate that the path exists and is a directory
    if !path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Path does not exist: {}", path.display()),
        ));
    }

    if !path.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Path is not a directory: {}", path.display()),
        ));
    }

    // Check for .lockdir file
    let lock_file_path = path.join(".lockdir");
    
    if !lock_file_path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Directory is not locked: {}", path.display()),
        ));
    }

    // Remove immutable attribute from the directory and its contents
    remove_immutable_attribute(&path)?;
    
    // Remove the lock file
    remove_file(lock_file_path)?;

    println!("Successfully unlocked directory: {}", path.display());
    Ok(())
}

// Remove immutable attribute from a path and its contents recursively
fn remove_immutable_attribute(path: &Path) -> Result<()> {
    // Make sure we're using the absolute path
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    
    // Run sudo chattr with the -i flag and -R for recursion
    let output = Command::new("sudo")
        .arg("chattr")
        .arg("-i")
        .arg("-R")
        .arg(abs_path)
        .output()?;
    
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(Error::new(
            ErrorKind::Other,
            format!("Failed to remove immutable attribute: {}", error_message),
        ));
    }
    
    Ok(())
}