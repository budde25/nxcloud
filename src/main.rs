#![forbid(unsafe_code)]

use crate::types::credentials::{Credentials, Password, Server, Username};
use crate::types::remote_path::RemotePathBuf;
use anyhow::{bail, Result};
use clap::AppSettings;
use log::{error, info, warn};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use xmltree::Element;

mod file;
mod http;
mod keyring;
mod types;
mod util;

/// Cli Enum for command parsing
#[derive(StructOpt)]
#[structopt(
    name = "NxCloud",
    about = "A command line client for interacting with your NextCloud server.",
    global_settings = &[AppSettings::ColoredHelp, AppSettings::InferSubcommands, AppSettings::VersionlessSubcommands, AppSettings::StrictUtf8]
)]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}
/// Main Cli struct with StructOpt
#[derive(Debug, StructOpt)]
#[structopt(
    name = "NxCloud",
    about = "A command line client for interacting with your NextCloud server."
)]
enum Command {
    /// Display's the account status.
    #[structopt(name = "status")]
    Status {},
    #[structopt(name = "login")]
    /// Login to your NextCloud server, please provide a app password for security.
    Login {
        /// The server url, Ex: https://cloud.example.com.
        #[structopt(parse(try_from_str))]
        server: Server,
        /// Your NextCloud username.
        #[structopt()]
        username: Username,
        /// A NextCloud app password, do not use your account password.
        #[structopt()]
        password: Password,
    },
    /// Logout of your NextCloud server.
    Logout,
    /// Push a file from your local machine to the server.
    #[structopt(name = "push")]
    Push {
        /// Path to source file.
        #[structopt(parse(from_os_str))]
        source: PathBuf,
        /// Path to destination file.
        #[structopt(parse(try_from_str))]
        destination: RemotePathBuf,
    },
    /// Pull a file from the server to your local machine.
    #[structopt(name = "pull")]
    Pull {
        /// Path to source file.
        #[structopt(parse(try_from_str))]
        source: RemotePathBuf,
        /// Path to destination file.
        #[structopt(parse(from_os_str))]
        destination: PathBuf,
    },

    /// List files and directories.
    #[structopt(name = "ls")]
    Ls {
        /// Path to source file.
        #[structopt(parse(try_from_str))]
        path: Option<RemotePathBuf>,

        #[structopt(short, long)]
        list: bool,

        #[structopt(short, long)]
        all: bool,
    },

    /// Make a directory.
    #[structopt(name = "mkdir")]
    Mkdir {
        /// Path to directory to create.
        #[structopt(parse(try_from_str))]
        path: RemotePathBuf,
    },

    /// Remove a file or directory, WARNING deletes files recursively.
    #[structopt(name = "rm")]
    Rm {
        /// Path to file or directory to remove.
        #[structopt(parse(try_from_str))]
        path: RemotePathBuf,

        // Force delete, will not show warning.
        #[structopt(short, long)]
        force: bool,
    },

    /// Enter an interactive prompt.
    #[structopt(name = "shell")]
    Shell {},

    /// Change directory of remote - Shell Only.
    #[structopt(name = "cd")]
    Cd {
        /// directory to change to.
        #[structopt(parse(try_from_str))]
        path: RemotePathBuf,
    },
}

/// Entrypoint of the program, returns 0 on success
fn main() -> Result<()> {
    //Command::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
    let current_dir = RemotePathBuf::from_str("/").unwrap();

    let cli = Opt::from_args();

    // Sets the log level
    match cli.verbose {
        0 => env_logger::builder()
            .filter_level(log::LevelFilter::Warn)
            .format_timestamp(None)
            .init(),
        1 => env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .format_timestamp(None)
            .init(),
        2 => env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .format_timestamp(None)
            .init(),
        _ => env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .format_timestamp(None)
            .init(),
    };

    info!("Logger has been initialized");

    run(cli, current_dir)?;

    Ok(())
}

fn run(cli: Opt, current_dir: RemotePathBuf) -> Result<RemotePathBuf> {
    let mut cur = current_dir.clone();
    match cli.cmd {
        Command::Status {} => status(),
        Command::Login { server, username, password } => {
            login(server, username, password)?
        }
        Command::Logout {} => logout()?,
        Command::Push { source, destination } => {
            push(source, current_dir.join(destination.as_path())?)?
        }
        Command::Pull { source, destination } => {
            pull(current_dir.join(source.as_path())?, destination)?
        }
        Command::Ls { path, list, all } => {
            let new_path = if let Some(remote_path) = path {
                current_dir.join(remote_path.as_path())?
            } else {
                current_dir
            };
            ls(new_path, list, all)?;
        }
        Command::Mkdir { path } => mkdir(current_dir.join(path.as_path())?)?,
        Command::Rm { path, force } => {
            rm(current_dir.join(path.as_path())?, force)?
        }
        Command::Shell {} => shell(current_dir)?,
        Command::Cd { path } => {
            cur = current_dir.join(path.as_path())?;
        }
    };
    Ok(cur)
}

/// Login to the NextCloud server
fn login(server: Server, username: Username, password: Password) -> Result<()> {
    let creds = Credentials::new(username, password, server);

    let http = creds.clone().into_http();
    http.get_user()?;
    creds.write()?;

    println!("Login successful");
    Ok(())
}

/// Logout of the NextCloud server
fn logout() -> Result<()> {
    match Credentials::delete() {
        Ok(_) => println!("Logout Successful"),
        Err(_) => bail!("Logout Failed"),
    }

    Ok(())
}

/// Prints the username and server of logged in user
fn status() {
    match Credentials::read() {
        Ok(creds) => {
            println!(
                "Logged in to Server: '{}' as User: '{}'",
                creds.server, creds.username
            );
        }
        Err(_) => println!("Not logged in"),
    }
}

/// lists files
fn ls(path: RemotePathBuf, list: bool, all: bool) -> Result<()> {
    // TODO fix this garbadge lol

    let creds = Credentials::read()?;
    let http = creds.into_http();
    let data: String = http.get_list(path.as_path())?;
    let xml = Element::parse(data.as_bytes()).unwrap();
    let items = xml.children;
    let mut files: Vec<String> = vec![];
    let mut full_path: Option<String> = None;
    for i in items {
        let resp = i.as_element().unwrap().to_owned().children;
        let file = resp[0].clone().as_element().unwrap().to_owned().children[0]
            .clone()
            .as_text()
            .unwrap()
            .to_owned();
        if full_path.is_none() {
            full_path = Some(file);
        } else {
            let a = full_path.clone().unwrap();
            let new_name = file.replace(&a, "").replace("%20", " ");
            if new_name.contains(' ') {
                files.push("'".to_owned() + &new_name + "'")
            } else if !new_name.starts_with('.') || all {
                files.push(new_name);
            }
        }
    }
    let print: String = if list { files.join("\n") } else { files.join("  ") };
    println!("{}", print);

    Ok(())
}

fn mkdir(path: RemotePathBuf) -> Result<()> {
    let creds = Credentials::read()?;
    creds.into_http().make_folder(&path.as_path())?;
    Ok(())
}

fn rm(path: RemotePathBuf, force: bool) -> Result<()> {
    if format!("{}", path) == "/" {
        error!("Deleting the root is not supported");
        return Ok(());
    }

    let warning = format!("Are you sure you want to delete '{}', (y/n)", path);

    if !force {
        warn!("DIRECTORIES DELETE ALL FILES AND DIRECTORIES RECURSIVELY");
        if !util::get_confirmation(&warning)? {
            return Ok(());
        }
    }

    let creds = Credentials::read()?;

    let http = creds.into_http();
    http.delete(path.as_path())?;
    Ok(())
}

/// Pulls a file from the server to your computer
fn pull(source: RemotePathBuf, destination: PathBuf) -> Result<()> {
    let creds = Credentials::read()?;
    let http = creds.into_http();

    let new_dest =
        util::format_destination_pull(source.as_path(), &destination)?;
    //let new_src = util::format_source_pull(&source)?;

    let data: Vec<u8> = http.get_file(source.as_path())?;
    file::create_file(&new_dest, &data)?;

    println!("Pulled {:?}, {:?}", source, new_dest);
    Ok(())
}

/// Pushes a file from your computer to the server
fn push(source: PathBuf, destination: RemotePathBuf) -> Result<()> {
    let creds = Credentials::read()?;
    let http = creds.into_http();

    let data = if let Ok(bytes) = file::read_file(&source) {
        bytes
    } else {
        println!("Must specify a file");
        return Ok(());
    };

    let mut destination = destination;
    if !destination.is_file() {
        // Ok since it needs to be a file to get the data from it
        let source_file_name = source.file_name().unwrap();
        destination.set_file_name(source_file_name);
    }

    http.send_file(destination.as_path(), data)?;
    println!("Push {:?}, {:?}", source, destination);
    Ok(())
}

fn shell(mut current_dir: RemotePathBuf) -> Result<()> {
    let mut rl = Editor::<()>::new();
    let history_path: PathBuf = file::HISTORY_PATH.to_path_buf();
    if rl.load_history(&history_path).is_ok() {
        info!("loaded prompt history");
    }
    loop {
        let prompt = format!("[{}] >> ", current_dir);
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                if line.as_str().to_lowercase() == "exit" {
                    break;
                }

                rl.add_history_entry(line.as_str());
                let mut nxcloud: Vec<&str> =
                    if line.as_str().starts_with("nxcloud") {
                        vec![]
                    } else {
                        vec!["nxcloud"]
                    };
                let vec: Vec<&str> = line.split(' ').collect::<Vec<&str>>();
                nxcloud.extend(vec);
                let cli = match Opt::from_iter_safe(nxcloud) {
                    Ok(c) => c,
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };
                current_dir = run(cli, current_dir)?;
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(&history_path).unwrap();
    Ok(())
}
