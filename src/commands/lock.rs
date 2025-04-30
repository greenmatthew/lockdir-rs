use std::fs::{self, File};
use std::io::{Error, ErrorKind, Result, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn lock_directory(dir_path: Option<&str>) -> Result<()> {
    // Determine the directory path to lock
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

    // Create a .lockdir file to indicate lock status
    let lock_file_path = path.join(".lockdir");
    
    // Check if directory is already locked
    if lock_file_path.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("Directory is already locked: {}", path.display()),
        ));
    }

    // Create .lockdir file with a helpful message
    let mut file = File::create(&lock_file_path)?;
    writeln!(file, "This directory has been locked by the 'lockdir' tool.")?;
    writeln!(file, "The immutable attribute has been set on this directory and its contents.")?;
    writeln!(file)?;
    writeln!(file, "To unlock this directory:")?;
    writeln!(file, "  - Use the lockdir tool: lockdir unlock <DIR>")?;
    writeln!(file, "  - Or manually: sudo chattr -i -R <DIR>")?;
    writeln!(file)?;
    writeln!(file, "Where <DIR> is the path to this directory containing the .lockdir file.")?;
    writeln!(file, "For more information, run: lockdir --help")?;

    // Set immutable flag on the directory and its contents
    match set_immutable_attribute(&path) {
        Ok(()) => {
            println!("Successfully locked directory: {}", path.display());
            Ok(())
        },
        Err(e) => {
            // If chattr failed, try to clean up the lock file
            let _ = fs::remove_file(&lock_file_path);
            Err(e)
        }
    }
}

// Set immutable attribute on a path and its contents recursively
fn set_immutable_attribute(path: &Path) -> Result<()> {
    // Make sure we're using the absolute path
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    
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