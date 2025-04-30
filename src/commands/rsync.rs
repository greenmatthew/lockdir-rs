use std::io::{Error, ErrorKind, Result};
use std::process::Command;
use clap::{ArgMatches, Command as ClapCommand, Arg};

/// Build the rsync subcommand
pub fn build_subcommand() -> ClapCommand {
    ClapCommand::new("rsync")
        .about("Synchronize files to or from locked directories using rsync")
        .arg(
            Arg::new("RSYNC_ARGS")
                .help("Arguments to pass directly to rsync (must be quoted)")
                .required(true)
                .trailing_var_arg(true)  // This is important for handling flags
                .allow_hyphen_values(true)  // Allow arguments starting with hyphens
                .value_parser(clap::value_parser!(String)),
        )
}

/// Handle the rsync subcommand
pub fn handle_command(sub_matches: &ArgMatches) -> Result<()> {
    // Get required rsync args
    let args = sub_matches.get_one::<String>("RSYNC_ARGS").expect("RSYNC_ARGS is required");
    
    println!("Running rsync with arguments: {args}");
    
    // Execute rsync as a shell command to preserve the original argument structure
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("rsync {args}"))
        .output()?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(Error::new(
            ErrorKind::Other,
            format!("Error running rsync: {error_message}"),
        ));
    }
    
    println!("Rsync completed successfully.");
    
    // Print stdout if any
    if !output.stdout.is_empty() {
        println!("Output:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    
    Ok(())
}