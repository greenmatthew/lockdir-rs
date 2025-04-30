use std::fs::remove_file;
use std::io::{Error, ErrorKind, Result};
use std::fmt::Write as FmtWrite;
use clap::{ArgMatches, Command, Arg, ArgAction};

use crate::fs_utils;

/// Build the unlock subcommand
pub fn build_subcommand() -> Command {
    Command::new("unlock")
        .about("Unlock a previously locked directory")
        .arg(
            Arg::new("PATHS")
                .help("Paths to the directories to unlock (defaults to current directory)")
                .required(false)
                .num_args(1..)
                .index(1),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .help("Force unlock even if some files seem to remain locked")
                .action(ArgAction::SetTrue),
        )
}

/// Handle the unlock subcommand
pub fn handle_command(sub_matches: &ArgMatches) -> Result<()> {
    // Using map_or_else instead of map + unwrap_or_else
    let paths: Vec<&str> = sub_matches
        .get_many::<String>("PATHS")
        .map_or_else(Vec::new, |vals| vals.map(String::as_str).collect());
    
    let force = sub_matches.get_flag("force");
    
    unlock_directories(&paths, force)
}

/// Unlock multiple directories
/// 
/// If paths is empty, the current directory will be unlocked
fn unlock_directories(paths: &[&str], force: bool) -> Result<()> {
    if paths.is_empty() {
        // If no paths provided, unlock the current directory
        return unlock_directory(None, force);
    }

    // Process each path
    let mut errors = Vec::new();
    
    for &path in paths {
        if let Err(err) = unlock_directory(Some(path), force) {
            // Collect errors but continue processing other paths
            errors.push((path, err));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        // Format all errors into a single error message
        let mut error_msg = String::new();
        error_msg.push_str("Failed to unlock one or more directories:\n");
        
        for (path, err) in errors {
            // Using write! macro to avoid allocation with format!
            let _ = writeln!(error_msg, "  - {path}: {err}");
        }
        
        Err(Error::new(ErrorKind::Other, error_msg))
    }
}

fn unlock_directory(dir_path: Option<&str>, force: bool) -> Result<()> {
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