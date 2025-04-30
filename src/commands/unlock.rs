use std::fs::remove_file;
use std::io::{Error, ErrorKind, Result};

use crate::fs_utils;

pub fn unlock_directory(dir_path: Option<&str>) -> Result<()> {
    // Validate the directory path
    let path = fs_utils::validate_directory(dir_path)?;

    // Check for .lockdir file
    let lock_file_path = path.join(".lockdir");
    
    if !lock_file_path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Directory is not locked: {}", path.display()),
        ));
    }

    // Remove immutable attribute from the directory and its contents
    fs_utils::remove_immutable_attribute(&path)?;
    
    // Remove the lock file
    remove_file(lock_file_path)?;

    println!("Successfully unlocked directory: {}", path.display());
    Ok(())
}