use anyhow::anyhow;
use bytes::Bytes;
use log::info;
use std::path::PathBuf;
use std::path::Path;
use structopt::StructOpt;
use url::ParseError;
use url::Url;
use xmltree::Element;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use dirs;
use path_dedot::ParseDot;

mod file;
mod http;
mod keyring;
mod util;

//// Structure for storing user credentials
#[derive(Debug)]
pub struct Creds {
    pub username: String,
    pub password: String,
    pub server: Url,
}

impl Creds {
    /// Returns a Result with Creds object or ParseError if an invalid url is supplied
    ///
    /// # Arguments
    /// `username` - String slice that represent a nextcloud login username  
    /// `password` - Sring slice that represents a nextcloud app password  
    /// `server` - String slice that represents a nextcloud server url, http or https can be omitted, https is the default
    ///
    /// # Examples
    /// ```
    /// let creds = Creds::new("user", "pass", "www.example.com")
    /// let creds = Creds::new("user", "pass", "https://www.example.com")
    /// ```
    fn new(username: &str, password: &str, server: &str) -> Result<Creds, ParseError> {
        let fqdn: String = if server.contains("https://") || server.contains("http://") {
            String::from(server)
        } else {
            format!("https://{}", server.to_string())
        };

        let url: Url = Url::parse(&fqdn)?;
        Ok(Creds {
            username: String::from(username),
            password: String::from(password),
            server: url,
        })
    }
}

/// Cli Enum for command parsing
#[derive(StructOpt)]
#[structopt(
    name = "NxCloud",
    about = "A command line client for interacting with your NextCloud server."
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
        #[structopt(parse(try_from_str = parse_url))]
        server: Url,
        /// Your NextCloud username.
        #[structopt()]
        username: String,
        /// A NextCloud app password, do not use your account password.
        #[structopt()]
        password: String,
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
        #[structopt(parse(from_os_str))]
        destination: PathBuf,
    },
    /// Pull a file from the server to your local machine.
    #[structopt(name = "pull")]
    Pull {
        /// Path to source file.
        #[structopt(parse(from_os_str))]
        source: PathBuf,
        /// Path to destination file.
        #[structopt(parse(from_os_str))]
        destination: PathBuf,
    },

    /// List files and directories.
    #[structopt(name = "ls")]
    Ls {
        /// Path to source file.
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,

        #[structopt(short, long)]
        list: bool,

        #[structopt(short, long)]
        all: bool,
    },

    /// Make a directory.
    #[structopt(name = "mkdir")]
    Mkdir {
        /// Path to directory to create.
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },

    /// Remove a file or directory, WARNING deletes files recursively.
    #[structopt(name = "rm")]
    Rm {
        /// Path to file or directory to remove.
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },

    /// Enter an interactive prompt.
    #[structopt(name = "shell")]
    Shell{},

    /// Change directory of remote - Shell Only.
    #[structopt(name = "cd")]
    Cd{
        /// directory to change to.
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}

/// Entrypoint of the program, returns 0 on success
fn main() -> anyhow::Result<()> {
    //Command::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
    let current_dir = PathBuf::from("/");

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

fn run(cli: Opt, mut current_dir: PathBuf) -> anyhow::Result<PathBuf> {

    match cli.cmd {
        Command::Status {} => status(),
        Command::Login {
            server,
            username,
            password,
        } => login(server, username, password)?,
        Command::Logout {} => logout()?,
        Command::Push {
            source,
            destination,
        } => push(source, destination, current_dir.clone())?,
        Command::Pull {
            source,
            destination,
        } => pull(source, destination, current_dir.clone())?,
        Command::Ls { path, list, all } => ls(path, current_dir.clone(), list, all)?,
        Command::Mkdir {path} => mkdir(path, current_dir.clone())?,
        Command::Rm {path} => rm(path, current_dir.clone())?,
        Command::Shell {} => shell(current_dir.clone())?,
        Command::Cd{path} => current_dir = cd(path, current_dir)?,
    };
    Ok(current_dir)
}

/// Login to the nextcloud server
fn login(server: Url, username: String, password: String) -> anyhow::Result<()> {
    let creds = Creds::new(&username, &password, &server.to_string())?;

    http::get_user(&creds)?;
    keyring::set_creds("username", &creds)?;

    println!("Login successful");
    Ok(())
}

/// Logout of the nextcloud server
fn logout() -> anyhow::Result<()> {
    match keyring::delete_creds("username") {
        Ok(_) => println!("Logout Successful"),
        Err(_) => return Err(anyhow!("Logout Failed")),
    }
    Ok(())
}

/// Prints the username and server of logged in user
fn status() {
    match keyring::get_creds("username") {
        Ok(creds) => {
            let username: String = creds.username;
            let server: Url = creds.server;

            println!("Logged in as {} for server {}", username, server);
        }
        Err(_) => println!("Not logged in"),
    }
}

/// lists files
fn ls(path: Option<PathBuf>, current_dir: PathBuf, list: bool, all: bool) -> anyhow::Result<()> {
    // TODO fix this garbadge lol
    let fp = if path.is_none() {
        current_dir
    } else {
        current_dir.join(path.unwrap())
    };

    let creds: Creds = keyring::get_creds("username")?;
    let data: String = http::get_list(&creds, &fp)?;
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
            if new_name.contains(" ") {
                files.push("'".to_owned() + &new_name + "'")
            } else if !new_name.starts_with('.') || all {
                files.push(new_name);
            }
        }
    }
    let print: String = if list {
        files.join("\n")
    } else {
        files.join("  ")
    };
    println!("{}", print);

    Ok(())
}

fn mkdir(path: PathBuf, current_dir: PathBuf) -> anyhow::Result<()> {
    let creds: Creds = keyring::get_creds("username")?;
    
    http::make_folder(&creds, &current_dir.join(path))?;
    Ok(())
}

fn rm(_path: PathBuf, current_dir: PathBuf) -> anyhow::Result<()> {
    println!("Not implemented yet, wan't to make it safe");
    Ok(())
}

/// Pulls a file from the server to your computer
fn pull(source: PathBuf, destination: PathBuf, current_dir: PathBuf) -> anyhow::Result<()> {
    let creds: Creds = keyring::get_creds("username")?;

    let full_source = current_dir.join(source);
    let new_dest = util::format_destination_pull(&full_source, &destination)?;
    let new_src = util::format_source_pull(&full_source)?;

    let data: Bytes = http::get_file(&creds, &new_src)?;
    file::create_file(&new_dest, &data)?;

    println!("Pulled {:?}, {:?}", new_src, new_dest);
    Ok(())
}

/// Pushes a file from your computer to the server
fn push(source: PathBuf, destination: PathBuf, current_dir: PathBuf) -> anyhow::Result<()> {
    let creds: Creds = keyring::get_creds("username")?;

    let data: Bytes = file::read_file(&source)?;
    let new_dest = util::format_destination_push(&source, &current_dir.join(destination))?;

    http::send_file(&creds, &new_dest, data)?;

    println!("Push {:?}, {:?}", source, new_dest);
    Ok(())
}

fn shell(mut current_dir: PathBuf) -> anyhow::Result<()>{
    let mut rl = Editor::<()>::new();
    let hist_path = dirs::home_dir().unwrap().join(".cache/nxcloud_history.txt");
    rl.load_history(&hist_path);
    loop {
        let prompt = format!("[{}] >> ", current_dir.to_string_lossy());
        let readline = rl.readline( &prompt);
        match readline {
            Ok(line) => {
                if line.as_str().to_lowercase() == "exit" {
                    break;
                }

                rl.add_history_entry(line.as_str());
                let mut nxcloud: Vec<&str> = if line.as_str().starts_with("nxcloud") { vec![] } else { vec!["nxcloud"] };
                let vec: Vec<&str> = line.split(" ").collect::<Vec<&str>>();
                nxcloud.extend(vec);
                let cli = match Opt::from_iter_safe(nxcloud) {
                    Ok(c) => c,
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };
                current_dir = run(cli, current_dir.to_path_buf())?;
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(&hist_path).unwrap();
    Ok(())
}

fn cd(path: PathBuf, current_dir: PathBuf) -> anyhow::Result<PathBuf> {
    // Overide dot methods cause they tend to fail
    if path.to_str().is_some() && path.to_str().unwrap() == "." {
        return Ok(current_dir);
    }
    if path.to_str().is_some() && path.to_str().unwrap() == ".." {
        return match current_dir.parent() {
            Some(p) => Ok(p.to_path_buf()),
            None => Ok(PathBuf::from("/"))
        }
    }

    if path.starts_with("/") {
        match path.parse_dot() {
            Ok(p) => Ok(p.to_path_buf()),
            Err(_) => Ok(PathBuf::from("/"))
        }
    } else {
        match current_dir.join(path).parse_dot() {
            Ok(p) => Ok(p.to_path_buf()),
            Err(_) => Ok(PathBuf::from("/"))
        }
    }
}

fn parse_url(src: &str) -> Result<Url, ParseError> {
    if src.contains("http") {
        Url::parse(src)
    } else {
        Url::parse(&("https://".to_owned() + src))
    }
}
