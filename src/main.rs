use anyhow::anyhow;
use bytes::Bytes;
use file::DEFAULT_PATH;
use std::path::PathBuf;
use structopt::StructOpt;
use url::ParseError;
use url::Url;

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
#[derive(Debug, StructOpt)]
#[structopt(
    name = "nxcloud",
    about = "A command line client for interacting with your NextCloud server."
)]
enum Cli {
    /// Display's the account status.
    #[structopt(name = "status")]
    Status {},
    #[structopt(name = "login")]
    /// Login to your NextCloud server, please provide a app password for security.
    Login {
        /// The server url, Ex: https://cloud.example.com.
        #[structopt(short = "s", long)]
        server: String,
        /// Your NextCloud username.
        #[structopt(short = "u", long)]
        username: String,
        /// A NextCloud app password, do not use your account password.
        #[structopt(short = "p", long)]
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
}

/// Entrypoint of the program, returns 0 on success
fn main() -> anyhow::Result<()> {
    let cli = Cli::from_args();
    match cli {
        Cli::Status {} => status(),
        Cli::Login {
            server,
            username,
            password,
        } => login(server, username, password)?,
        Cli::Logout {} => logout()?,
        Cli::Push {
            source,
            destination,
        } => push(source, destination)?,
        Cli::Pull {
            source,
            destination,
        } => pull(source, destination)?,
    };
    return Ok(());
}

/// Login to the nextcloud server
fn login(server: String, username: String, password: String) -> anyhow::Result<()> {
    let creds = Creds::new(&username, &password, &server)?;

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
