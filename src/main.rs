use clap::{Command, Arg};
mod commands;

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
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("license") {
        println!("{LICENSE}");
        return;
    }

    match matches.subcommand() {
        _ => {
            panic!("Unexpected command - This should not be reached due to arg_required_else_help(true)");
        }
    }
}