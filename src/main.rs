use anyhow::anyhow;
use bytes::Bytes;
use std::path::PathBuf;
use structopt::clap::Shell;
use structopt::StructOpt;
use url::ParseError;
use url::Url;
use log::{info};
use xmltree;

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
    /// `server` - String slice that represents a nextcloud server url, http or https can be ommited, https is the default  
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
        return Ok(Creds {
            username: String::from(username),
            password: String::from(password),
            server: url,
        });
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
    /// Pull a file from the server to your local machine.
    #[structopt(name = "ls")]
    Ls {
        /// Path to source file.
        #[structopt(parse(from_os_str))]
        path: PathBuf,

        #[structopt(short, long)]
        list: bool,

        #[structopt(short, long)]
        all: bool,
    },
}

/// Entrypoint of the program, returns 0 on success
fn main() -> anyhow::Result<()> {
    //Command::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");

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
        } => push(source, destination)?,
        Command::Pull {
            source,
            destination,
        } => pull(source, destination)?,
        Command::Ls {
            path,
            list,
            all
        } => ls(path, list, all)?
    };
    return Ok(());
}

/// Login to the nextcloud server
fn login(server: Url, username: String, password: String) -> anyhow::Result<()> {
    let creds = Creds::new(&username, &password, &server.to_string())?;

    http::get_user(&creds)?;
    keyring::set_creds("username", &creds)?;

    println!("Login successful");
    return Ok(());
}

/// Logout of the nextcloud server
fn logout() -> anyhow::Result<()> {
    match keyring::delete_creds("username") {
        Ok(_) => println!("Logout Successful"),
        Err(_) => return Err(anyhow!("Logout Failed")),
    }
    return Ok(());
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

fn ls(path: PathBuf, list: bool, all: bool) -> anyhow::Result<()> {
    // TODO fix this garbadge lol
    let creds: Creds = keyring::get_creds("username")?;
    let data: String = http::get_list(&creds, &path)?;
    let xml = xmltree::Element::parse(data.as_bytes()).unwrap();
    let items = xml.children;
    let mut files: Vec<String> = vec![]; 
    let mut fullpath: Option<String> = None;
    for i in items {
        let resp = i.as_element().unwrap().to_owned().children;
        let file = resp[0].clone().as_element().unwrap().to_owned().children[0].clone().as_text().unwrap().to_owned();
        if fullpath.is_none() {
            fullpath = Some(file);
        } else {
            let a = fullpath.clone().unwrap();
            let new_name = file.replace(&a, "").replace("%20", " ");
            if new_name.contains(" ") {
                files.push("'".to_owned() + &new_name + "'")
            } else {
                if !new_name.starts_with(".") || all {
                    files.push(new_name);
                }
            }
        }
    }
    let print: String = if list { files.join("\n") } else { files.join("  ") };
    println!("{}", print);

    Ok(())
}

/// Pulls a file from the server to your computer
fn pull(source: PathBuf, destination: PathBuf) -> anyhow::Result<()> {
    let creds: Creds = keyring::get_creds("username")?;

    let new_dest = util::format_destination_pull(&source, &destination)?;
    let new_src = util::format_source_pull(&source)?;

    let data: Bytes = http::get_file(&creds, &new_src)?;
    file::create_file(&new_dest, &data)?;

    println!("Pulled {:?}, {:?}", new_src, new_dest);
    return Ok(());
}

/// Pushes a file from your computer to the server
fn push(source: PathBuf, destination: PathBuf) -> anyhow::Result<()> {
    let creds: Creds = keyring::get_creds("username")?;

    let data: Bytes = file::read_file(&source)?;
    let new_dest = util::format_destination_push(&source, &destination)?;

    http::send_file(&creds, &new_dest, data)?;

    println!("Push {:?}, {:?}", source, new_dest);
    return Ok(());
}

fn parse_url(src: &str) -> Result<Url, ParseError> {
    if src.contains("http") {
        Url::parse(src)
    } else {
        Url::parse(&("https://".to_owned() + src))
    }
}