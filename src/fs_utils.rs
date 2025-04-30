use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Set immutable attribute on a path and its contents recursively
pub fn set_immutable_attribute(path: &Path) -> Result<()> {
    let abs_path = get_absolute_path(path)?;
    
    // Run sudo chattr with the +i flag and -R for recursion
    let output = Command::new("sudo")
        .arg("chattr")
        .arg("+i")
        .arg("-R")
        .arg(abs_path)
        .output()?;
    
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(Error::new(
            ErrorKind::Other,
            format!("Failed to set immutable attribute: {error_message}"),
        ));
    }
    
    Ok(())
}

/// Check if the immutable attribute is set on a path
pub fn check_immutable_attribute(path: &Path) -> Result<bool> {
    let abs_path = get_absolute_path(path)?;
    
    // Run lsattr to check the attributes on the path
    let output = Command::new("lsattr")
        .arg("-d")  // Only list directory, not its contents
        .arg(abs_path)
        .output()?;
    
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(Error::new(
            ErrorKind::Other,
            format!("Failed to check immutable attribute: {error_message}"),
        ));
    }
    
    // Parse the output to see if the immutable flag is set
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // The output format is like: "----i---------- /path/to/dir"
    // We need to check if 'i' appears in the attribute string
    Ok(output_str.contains("----i") || output_str.contains(" i "))
}

/// Remove immutable attribute from a path and its contents recursively
pub fn remove_immutable_attribute(path: &Path) -> Result<()> {
    let abs_path = get_absolute_path(path)?;
    
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
            format!("Failed to remove immutable attribute: {error_message}"),
        ));
    }
    
    Ok(())
}

/// Helper function to ensure we're using absolute paths
fn get_absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

/// Check if a directory is valid for lock/unlock operations
pub fn validate_directory(dir_path: Option<&str>) -> Result<PathBuf> {
    // Determine the directory path
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

    Ok(path)
}