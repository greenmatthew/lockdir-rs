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
                    Arg::new("PATH")
                        .help("Path to the directory to lock (defaults to current directory)")
                        .required(false)
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
                    Arg::new("PATH")
                        .help("Path to the directory to unlock (defaults to current directory)")
                        .required(false)
                        .index(1),
                )
        )
        .get_matches();

    if matches.get_flag("license") {
        println!("{LICENSE}");
        return;
    }

    match matches.subcommand() {
        Some(("lock", sub_matches)) => {
            let path = sub_matches.get_one::<String>("PATH").map(String::as_str);
            let force = sub_matches.get_flag("force");
            
            if let Err(err) = commands::lock::lock_directory(path, force) {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
        Some(("unlock", sub_matches)) => {
            let path = sub_matches.get_one::<String>("PATH").map(String::as_str);
            
            if let Err(err) = commands::unlock::unlock_directory(path) {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
        _ => {
            panic!("Unexpected command - This should not be reached due to arg_required_else_help(true)");
        }
    }
}