use clap::{Command, Arg, ArgAction};
mod commands;
mod fs_utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LICENSE: &str = include_str!("../LICENSE");

fn main() {
    let matches = Command::new("lockdir")
        .version(VERSION)
        .about("File-based directory locking utility")
        .arg_required_else_help(true)
        .arg(
            Arg::new("license")
                .long("license")
                .help("Display the license information")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
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
        )
        .subcommand(
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
        )
        .subcommand(
            Command::new("rsync")
                .about("Synchronize files to or from locked directories using rsync")
                .arg(
                    Arg::new("SOURCE")
                        .help("Source path (file or directory)")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("DESTINATION")
                        .help("Destination path (file or directory)")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("move")
                        .short('m')
                        .long("move")
                        .help("Move files instead of copying (removes source files after successful transfer)")
                        .action(ArgAction::SetTrue),
                )
        )
        .get_matches();

    if matches.get_flag("license") {
        println!("{LICENSE}");
        return;
    }

    match matches.subcommand() {
        Some(("lock", sub_matches)) => {
            // Using map_or_else instead of map + unwrap_or_else
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("PATHS")
                .map_or_else(Vec::new, |vals| vals.map(String::as_str).collect());
            
            let force = sub_matches.get_flag("force");
            
            if let Err(err) = commands::lock::lock_directories(&paths, force) {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
        Some(("unlock", sub_matches)) => {
            // Using map_or_else instead of map + unwrap_or_else
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("PATHS")
                .map_or_else(Vec::new, |vals| vals.map(String::as_str).collect());
            
            let force = sub_matches.get_flag("force");
            
            if let Err(err) = commands::unlock::unlock_directories(&paths, force) {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
        Some(("rsync", sub_matches)) => {
            // Get required source and destination arguments
            let source = sub_matches.get_one::<String>("SOURCE").expect("SOURCE is required");
            let destination = sub_matches.get_one::<String>("DESTINATION").expect("DESTINATION is required");
            
            let move_files = sub_matches.get_flag("move");
            
            if let Err(err) = commands::rsync::rsync_directories(source, destination, move_files) {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
        _ => {
            panic!("Unexpected command - This should not be reached due to arg_required_else_help(true)");
        }
    }
}