use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};

use crate::fs_utils;

pub fn lock_directory(dir_path: Option<&str>) -> Result<()> {
    // Validate the directory path
    let path = fs_utils::validate_directory(dir_path)?;

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