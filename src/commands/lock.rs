use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};
use std::fmt::Write as FmtWrite;
use clap::{ArgMatches, Command, Arg, ArgAction};

use crate::fs_utils;

/// Build the lock subcommand
pub fn build_subcommand() -> Command {
    Command::new("lock")
        .about("Lock a directory by setting the immutable attribute")
        .arg(
            Arg::new("PATHS")
                .help("Paths to the directories to lock (defaults to current directory)")
                .required(false)
                .num_args(1..)
                .index(1),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .help("Force lock even if directory appears to be already locked")
                .action(ArgAction::SetTrue),
        )
}

/// Handle the lock subcommand
pub fn handle_command(sub_matches: &ArgMatches) -> Result<()> {
    // Using map_or_else instead of map + unwrap_or_else
    let paths: Vec<&str> = sub_matches
        .get_many::<String>("PATHS")
        .map_or_else(Vec::new, |vals| vals.map(String::as_str).collect());
    
    let force = sub_matches.get_flag("force");
    
    lock_directories(&paths, force)
}

/// Lock multiple directories
/// 
/// If paths is empty, the current directory will be locked
fn lock_directories(paths: &[&str], force: bool) -> Result<()> {
    if paths.is_empty() {
        // If no paths provided, lock the current directory
        return lock_directory(None, force);
    }

    // Process each path
    let mut errors = Vec::new();
    
    for &path in paths {
        if let Err(err) = lock_directory(Some(path), force) {
            // Collect errors but continue processing other paths
            errors.push((path, err));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        // Format all errors into a single error message
        let mut error_msg = String::new();
        error_msg.push_str("Failed to lock one or more directories:\n");
        
        for (path, err) in errors {
            // Using write! macro to avoid allocation with format!
            let _ = writeln!(error_msg, "  - {path}: {err}");
        }
        
        Err(Error::new(ErrorKind::Other, error_msg))
    }
}

fn lock_directory(dir_path: Option<&str>, force: bool) -> Result<()> {
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
            println!("Directory is already locked, but proceeding with force lock anyway.");
            
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