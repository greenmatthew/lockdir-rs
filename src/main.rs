use clap::{Command, Arg, ArgAction};
use std::process;

mod commands;
mod fs_utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LICENSE: &str = include_str!("../LICENSE");

fn main() {
    let matches = build_cli().get_matches();

    if matches.get_flag("license") {
        println!("{LICENSE}");
        return;
    }

    // Match subcommands and execute them
    let result = match matches.subcommand() {
        Some(("lock", sub_matches)) => {
            commands::lock::handle_command(sub_matches)
        }
        Some(("unlock", sub_matches)) => {
            commands::unlock::handle_command(sub_matches)
        }
        Some(("rsync", sub_matches)) => {
            commands::rsync::handle_command(sub_matches)
        }
        _ => {
            panic!("Unexpected command - This should not be reached due to arg_required_else_help(true)");
        }
    };

    // Handle any errors
    if let Err(err) = result {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

/// Build the main CLI
fn build_cli() -> Command {
    Command::new("lockdir")
        .version(VERSION)
        .about("File-based directory locking utility")
        .arg_required_else_help(true)
        .arg(
            Arg::new("license")
                .long("license")
                .help("Display the license information")
                .action(ArgAction::SetTrue),
        )
        .subcommand(commands::lock::build_subcommand())
        .subcommand(commands::unlock::build_subcommand())
        .subcommand(commands::rsync::build_subcommand())
}