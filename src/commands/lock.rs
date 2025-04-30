use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};

use crate::fs_utils;
use crate::commands::unlock;

pub fn lock_directory(dir_path: Option<&str>, force: bool) -> Result<()> {
    // Validate the directory path
    let path = fs_utils::validate_directory(dir_path)?;

    // Create a .lockdir file to indicate lock status
    let lock_file_path = path.join(".lockdir");
    
    // Check if directory is already locked
    if lock_file_path.exists() {
        if force {
            // If force is true, show message and proceed with force locking
            println!("Directory is already locked... forcing it locked anyways.");
            
            // Try to unlock the directory first
            match unlock::unlock_directory(dir_path) {
                Ok(()) => {
                    // Success silently continues to locking
                }
                Err(e) => {
                    // Only log warning if unlock fails but still continue
                    eprintln!("Warning: Could not properly unlock directory: {e}");
                }
            }
        } else {
            // Without force flag, return an error with hint to use force flag
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("Directory is already locked: {} (try using -f to force it to lock the dir and all contents anyway)", path.display()),
            ));
        }
    } else if force {
        // If force is set but no lockfile exists, check if we can actually set attributes
        // This would handle the case where directory might be locked but .lockdir is missing
        let test_result = fs_utils::check_immutable_attribute(&path);
        if test_result.is_err() || test_result.unwrap() {
            println!(".lockdir file is missing, but directory may be locked. Replacing lock file and re-locking contents.");
            
            // Attempt to remove immutable attribute first
            let _ = fs_utils::remove_immutable_attribute(&path);
        }
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
    match fs_utils::set_immutable_attribute(&path) {
        Ok(()) => {
            println!("Successfully locked directory: {}", path.display());
            Ok(())
        },
        Err(e) => {
            // If chattr failed, try to clean up the lock file
            let _ = std::fs::remove_file(&lock_file_path);
            Err(e)
        }
    }
}