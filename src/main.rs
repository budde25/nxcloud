use anyhow::anyhow;
use bytes::Bytes;
use clap::AppSettings;
use log::{error, info, warn};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use url::{ParseError, Url};
use xmltree::Element;

mod file;
mod http;
mod keyring;
mod util;

//// Structure for storing user credentials
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: Username,
    pub password: Password,
    pub server: Server,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Username(String);

#[derive(Clone, PartialEq)]
pub struct Password(String);

#[derive(Debug, Clone, PartialEq)]
pub struct Server(Url);

impl Username {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Username {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Password {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Password").field(self).finish()
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Password {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Server {
    pub fn new(u: Url) -> Result<Self, ParseError> {
        let mut u = u;
        if u.set_scheme("https").is_err() {
            return Err(ParseError::RelativeUrlWithoutBase);
        }
        Ok(Self(u))
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Server {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let u = if s.starts_with("http://") || s.starts_with("https://") {
            Url::parse(s)?
        } else {
            let fqdn = format!("{}{}", "https://", s);
            Url::parse(&fqdn)?
        };
        Self::new(u)
    }
}

impl Credentials {
    /// Returns a Result with Credentials object or ParseError if an invalid url is supplied
    ///
    /// # Arguments
    /// `username` - String slice that represent a NextCloud login username  
    /// `password` - String slice that represents a NextCloud app password  
    /// `server` - String slice that represents a NextCloud server url, http or https can be omitted, https is the default
    ///
    /// # Examples
    /// ```
    /// let creds = Credentials::parse("user", "pass", "www.example.com")
    /// let creds = Credentials::parse("user", "pass", "https://www.example.com")
    /// ```
    fn parse(
        username: &str,
        password: &str,
        server: &str,
    ) -> anyhow::Result<Self> {
        let username = Username::new(username.to_string());
        let password = Password::new(password.to_string());
        let server = Server::from_str(server)?;
        Ok(Credentials::new(username, password, server))
    }

    fn new(username: Username, password: Password, server: Server) -> Self {
        Self { username, password, server }
    }
}
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
        Command::Login { server, username, password } => {
            login(server, username, password)?
        }
        Command::Logout {} => logout()?,
        Command::Push { source, destination } => push(
            source,
            util::join_dedot_path(current_dir.clone(), destination)?,
        )?,
        Command::Pull { source, destination } => pull(
            util::join_dedot_path(current_dir.clone(), source)?,
            destination,
        )?,
        Command::Ls { path, list, all } => {
            let fp = if path.is_none() {
                current_dir.clone()
            } else {
                util::join_dedot_path(current_dir.clone(), path.unwrap())?
            };
            ls(fp, list, all)?;
        }
        Command::Mkdir { path } => {
            mkdir(util::join_dedot_path(current_dir.clone(), path)?)?
        }
        Command::Rm { path, force } => {
            rm(util::join_dedot_path(current_dir.clone(), path)?, force)?
        }
        Command::Shell {} => shell(current_dir.clone())?,
        Command::Cd { path } => {
            current_dir = util::join_dedot_path(current_dir.clone(), path)?
        }
    };
    Ok(current_dir)
}

/// Login to the nextcloud server
fn login(
    server: Server,
    username: Username,
    password: Password,
) -> anyhow::Result<()> {
    let creds = Credentials::new(username, password, server);

    let http = creds.clone().into_http();
    http.get_user()?;
    creds.write()?;

    println!("Login successful");
    Ok(())
}

/// Logout of the nextcloud server
fn logout() -> anyhow::Result<()> {
    match Credentials::delete() {
        Ok(_) => println!("Logout Successful"),
        Err(_) => return Err(anyhow!("Logout Failed")),
    }

    Ok(())
}

/// Prints the username and server of logged in user
fn status() {
    match Credentials::read() {
        Ok(creds) => {
            let username = creds.username;
            let server = creds.server;
            println!(
                "Logged in to Server: '{}' as User: '{}'",
                server, username
            );
        }
        Err(_) => println!("Not logged in"),
    }
}

/// lists files
fn ls(path: PathBuf, list: bool, all: bool) -> anyhow::Result<()> {
    // TODO fix this garbadge lol

    let creds = Credentials::read()?;
    let http = creds.into_http();
    let data: String = http.get_list(&path)?;
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

fn mkdir(path: PathBuf) -> anyhow::Result<()> {
    let creds = Credentials::read()?;
    creds.into_http().make_folder(&path)?;
    Ok(())
}

fn rm(path: PathBuf, force: bool) -> anyhow::Result<()> {
    if path.to_string_lossy() == "/" {
        error!("Deleting the root is not supported");
        return Ok(());
    }

    let warning = format!(
        "Are you sure you want to delete '{}', (y/n)",
        path.to_string_lossy()
    );

    if !force {
        warn!("DIRECTORIES DELETE ALL FILES AND DIRECTORIES RECURSIVELY");
        if !util::get_confirmation(&warning)? {
            return Ok(());
        }
    }

    let creds = Credentials::read()?;

    let http = creds.into_http();
    http.delete(&path)?;
    Ok(())
}

/// Pulls a file from the server to your computer
fn pull(source: PathBuf, destination: PathBuf) -> anyhow::Result<()> {
    let creds = Credentials::read()?;
    let http = creds.into_http();

    let new_dest = util::format_destination_pull(&source, &destination)?;
    let new_src = util::format_source_pull(&source)?;

    let data: Bytes = http.get_file(&new_src)?;
    file::create_file(&new_dest, &data)?;

    println!("Pulled {:?}, {:?}", new_src, new_dest);
    Ok(())
}

/// Pushes a file from your computer to the server
fn push(source: PathBuf, destination: PathBuf) -> anyhow::Result<()> {
    let creds = Credentials::read()?;
    let http = creds.into_http();

    let data: Bytes = file::read_file(&source)?;
    let new_dest = util::format_destination_push(&source, &destination)?;

    http.send_file(&new_dest, data)?;

    println!("Push {:?}, {:?}", source, new_dest);
    Ok(())
}

fn shell(mut current_dir: PathBuf) -> anyhow::Result<()> {
    let mut rl = Editor::<()>::new();
    let history_path: PathBuf = file::HISTORY_PATH.to_path_buf();
    if rl.load_history(&history_path).is_ok() {
        info!("loaded prompt history");
    }
    loop {
        let prompt = format!("[{}] >> ", current_dir.to_string_lossy());
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
                current_dir = run(cli, current_dir.to_path_buf())?;
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
