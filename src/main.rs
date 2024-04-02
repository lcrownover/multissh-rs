use anyhow::{bail, Result};
use clap::Parser;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// Blazingly Fast Parallel SSH
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Comma-separated list of target hostnames or IP addresses
    /// (e.g. "host1,host2,host3")
    #[clap(short, long)]
    targets: Option<String>,

    /// Path to a file containing a list of target hostnames or IP addresses to use as targets
    /// (default: ~/.config/multissh/targets; ~/.multissh/targets; /etc/multissh/targets)
    /// (e.g. "/path/to/targets.txt")
    #[clap(short = 'f', long)]
    targets_file: Option<PathBuf>,

    /// Path to a file containing an inventory of target hostnames or IP addresses
    /// (default: ~/.config/multissh/inventory; ~/.multissh/inventory; /etc/multissh/inventory)
    /// (e.g. "/path/to/inventory.yml")
    #[clap(short = 'i', long)]
    inventory_file: Option<PathBuf>,

    /// Name of an inventory group to use as targets
    /// (required if -i/--inventory-file is used)
    /// (e.g. "web-servers")
    #[clap(short = 'g', long)]
    inventory_group: Option<String>,

    /// Username to use when connecting to target hosts
    /// (default: $USER)
    #[clap(short, long)]
    user: Option<String>,

    /// Password to use when connecting to target hosts
    #[clap(short, long)]
    password: Option<String>,

    /// Ask for password
    #[clap(short = 'a', long)]
    ask_password: bool,

    /// Path to a private key to use when connecting to target hosts
    /// (default: ~/.ssh/id_rsa)
    #[clap(short = 'k', long, default_value = "~/.ssh/id_rsa")]
    private_key: Option<PathBuf>,

    /// Port to use when connecting to target hosts
    /// (default: 22)
    #[clap(short = 'P', long, default_value = "22")]
    port: Option<u16>,

    /// Timeout in seconds to wait for a connection to a target host
    /// (default: 10)
    #[clap(long, default_value = "10")]
    timeout: Option<u64>,

    /// Enable verbose output
    /// (default: false)
    #[clap(long)]
    verbose: bool,

    /// Command to run on target hosts
    /// (e.g. "uname -a")
    #[clap()]
    command: String,
}

struct Config {
    default_inventory_file: Vec<PathBuf>,
    default_private_key: Vec<PathBuf>,
    default_port: u16,
    default_timeout: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_inventory_file: vec![
                PathBuf::from("~/.multissh/inventory"),
                PathBuf::from("~/.config/multissh/inventory"),
                PathBuf::from("/etc/multissh/inventory"),
            ],
            default_private_key: vec![PathBuf::from("~/.ssh/id_rsa")],
            default_port: 22,
            default_timeout: 10,
        }
    }
}

fn read_targets_file(targets_file: &PathBuf) -> Result<Vec<String>> {
    // Read targets from file
    if Path::new(targets_file).exists() {
        let lines = std::fs::read_to_string(targets_file)?;
        return Ok(lines
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.starts_with("#"))
            .collect());
    }
    bail!("File not found: {}", targets_file.display());
}

fn read_inventory_file(inventory_file: &PathBuf, group: String) -> Result<Vec<String>> {
    // Read inventory from file
    if Path::new(inventory_file).exists() {
        let lines = std::fs::read_to_string(inventory_file)?;
        return Ok(lines
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.starts_with("#"))
            .collect());
    }
    bail!("File not found: {}", inventory_file.display());
}

trait OptionExt<T> {
    fn to_int(&self) -> i32;
}

impl<T> OptionExt<T> for Option<T> {
    fn to_int(&self) -> i32 {
        match self {
            Some(_) => 1,
            None => 0,
        }
    }
}

#[allow(dead_code)]
fn get_targets(cli: &Cli) -> Result<Vec<String>> {
    // If no target options were used, return an error
    // If more than one target option was used, return an error
    // If --targets was used, just return the targets as a vector of strings
    // If --targets-file was used, read the targets from the file. If -f is an empty string, use the default file path
    // If --inventory-file was used, read the inventory file and get the targets from the provided --inventory-group

    // Check if one of the target options was used
    if cli.targets.is_none() && cli.targets_file.is_none() && cli.inventory_group.is_none() {
        bail!("One of -t/--targets, -f/--targets-file, or -i/--inventory-file is required");
    }

    // Check if more than one target option was used
    if cli.targets.to_int() + cli.targets_file.to_int() + cli.inventory_group.to_int() > 1 {
        bail!("Only one of -t/--targets, -f/--targets-file, or -i/--inventory-file can be used");
    }

    // --targets was used
    // just return the targets as a vector of strings
    if let Some(targets) = &cli.targets {
        return Ok(targets.split(',').map(|s| s.to_string()).collect());
    }

    // --targets-file was used
    // read the targets from the file
    if let Some(targets_file) = &cli.targets_file {
        return match read_targets_file(targets_file) {
            Ok(targets) => Ok(targets),
            Err(e) => {
                bail!(
                    "Failed to use target file {}: {}",
                    targets_file.display(),
                    e
                );
            }
        };
    }

    // --inventory-file was used
    // read the inventory file and get the targets from the provided inventory group
    if let Some(inventory_file) = &cli.inventory_file {
        // Read targets from inventory
        unimplemented!()
    }

    bail!("One of -t/--targets, -f/--targets-file, or -i/--inventory-file is required");
}

fn main() -> Result<()> {
    // let msgs = vec!["Hello", "World", "from", "Rayon"];
    // msgs.par_iter().for_each(|msg| println!("{}", msg));
    let cli = Cli::parse();
    let targets = get_targets(&cli)?;
    targets.par_iter().for_each(|target| {
        println!("Running command on target: {}", target);
    });

    Ok(())
}

// Usage:
// multissh [OPTIONS] COMMAND
//  
//      ONE OF:
//  -t/--targets (comma-separated list of target hostnames or IP addresses)
//      OR
//  -f/--targets-file (default: ~/.config/multissh/targets; ~/.multissh/targets; /etc/multissh/targets)
//      OR
//  -i/--inventory-file (default: ~/.config/multissh/inventory; ~/.multissh/inventory; /etc/multissh/inventory)
//  -g/--inventory-group (required if -i/--inventory-file is used)
//
//      OTIONAL:
//  -u/--user (default: $USER)
//  -p/--password
//  -k/--private-key (default: ~/.ssh/id_rsa)
//  -P/--port (default: 22)
//  -t/--timeout (default: 10)
//  -v/--verbose (default: false)
//  -h/--help
//  -V/--version
