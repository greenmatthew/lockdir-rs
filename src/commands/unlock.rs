use std::fs::remove_file;
use std::io::{Error, ErrorKind, Result};

use crate::fs_utils;

pub fn unlock_directory(dir_path: Option<&str>, force: bool) -> Result<()> {
    // Validate the directory path
    let path = fs_utils::validate_directory(dir_path)?;

    // Check for .lockdir file
    let lock_file_path = path.join(".lockdir");
    
    // Check if directory is locked (either by lock file or immutable attribute)
    let is_immutable = match fs_utils::check_immutable_attribute(&path) {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to check directory lock status: {e}"),
            ));
        }
    };
    
    if !lock_file_path.exists() && !is_immutable {
        // Directory is not locked at all
        if force {
            println!("Directory is not locked, but proceeding with force unlock anyway.");
        } else {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Directory is not locked: {} (try using -f to force it to unlock the dir and all contents anyway)", path.display()),
            ));
        }
    } else if !lock_file_path.exists() && is_immutable {
        // Directory is immutable but .lockdir file is missing
        if force {
            println!("Directory appears to be locked but .lockdir file is missing. Forcing unlock...");
        } else {
            // Without force flag, return an error with hint to use force flag
            return Err(Error::new(
                ErrorKind::Other,
                format!("Directory appears to be locked but .lockdir file is missing: {} (try using -f to force it to unlock the dir and all contents anyway)", path.display()),
            ));
        }
    } else if lock_file_path.exists() && !is_immutable {
        // Lock file exists but directory is not immutable
        if force {
            println!(".lockdir file exists, but directory is not immutable. Cleaning up lock file.");
        } else {
            // Without force flag, return an error with hint to use force flag
            return Err(Error::new(
                ErrorKind::Other,
                format!("Directory has a lock file but is not immutable: {} (try using -f to force it to unlock the dir and all contents anyway)", path.display()),
            ));
        }
    }

    // Remove immutable attribute from the directory and its contents
    match fs_utils::remove_immutable_attribute(&path) {
        Ok(()) => {
            // Only try to remove the lock file if it exists
            if lock_file_path.exists() {
                if let Err(e) = remove_file(&lock_file_path) {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Failed to remove lock file: {e}"),
                    ));
                }
            }
            
            println!("Successfully unlocked directory: {}", path.display());
            Ok(())
        },
        Err(e) => {
            // If force is true, try to remove the lock file anyway
            if force {
                println!("Warning: Failed to remove immutable attribute: {e}");
                
                // Try to remove the lock file if it exists
                if lock_file_path.exists() {
                    if let Err(file_err) = remove_file(&lock_file_path) {
                        println!("Warning: Also failed to remove lock file: {file_err}");
                    } else {
                        println!("Successfully removed lock file, but directory may still have immutable attributes.");
                    }
                }
                
                println!("Force unlock attempted on directory: {}", path.display());
                println!("Some files may still be locked. You might need to run: sudo chattr -i -R {}", path.display());
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}