use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};

use crate::fs_utils;

pub fn lock_directory(dir_path: Option<&str>, force: bool) -> Result<()> {
    // Validate the directory path
    let path = fs_utils::validate_directory(dir_path)?;

    // Create a .lockdir file to indicate lock status
    let lock_file_path = path.join(".lockdir");
    
    // Check if directory is already locked (either by lock file or immutable attribute)
    let is_immutable = match fs_utils::check_immutable_attribute(&path) {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to check directory lock status: {e}"),
            ));
        }
    };
    
    if lock_file_path.exists() {
        if force {
            // If force is true, proceed with force locking silently
            
            // Try to unlock the directory first without messages
            let _ = fs_utils::remove_immutable_attribute(&path);
            // Remove the lock file silently if it exists
            let _ = std::fs::remove_file(&lock_file_path);
        } else {
            // Without force flag, return an error with hint to use force flag
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("Directory is already locked: {} (try using -f to force it to lock the dir and all contents anyway)", path.display()),
            ));
        }
    } else if is_immutable {
        // Directory is immutable but .lockdir file is missing
        if force {
            println!(".lockdir file is missing, but directory appears to be already locked. Replacing lock file and re-locking contents.");
            
            // Attempt to remove immutable attribute first
            let _ = fs_utils::remove_immutable_attribute(&path);
        } else {
            // Without force flag, we should inform the user that directory appears to be locked
            return Err(Error::new(
                ErrorKind::Other,
                format!("Directory appears to be locked but .lockdir file is missing: {} (use -f to force lock)", path.display()),
            ));
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