use bytes::Bytes;
use file::DEFAULT_PATH;
use std::ffi::OsStr;
use std::path::Path;
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
        let fqdn: String = if !server.contains("https://") || !server.contains("http://") {
            format!("https://{}", server.to_string())
        } else {
            String::from(server)
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
    name = "nextcloudcli",
    about = "A command line client for interacting with your NextCloud server."
)]
enum Cli {
    /// Display's the account status.
    #[structopt(name = "status")]
    Status {},
    #[structopt(name = "login")]
    /// Login to your NextCloud server, please provide a app password for security.
    Login {
        /// The server url, ex: https://cloud.example.com.
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
fn main() {
    let cli = Cli::from_args();
    match cli {
        Cli::Status {} => status(),
        Cli::Login {
            server,
            username,
            password,
        } => login(server, username, password),
        Cli::Logout {} => logout(),
        Cli::Push {
            source,
            destination,
        } => push(source, destination),
        Cli::Pull {
            source,
            destination,
        } => pull(source, destination),
    };
}

/// Login to the nextcloud server
fn login(server: String, username: String, password: String) {
    if let Ok(creds) = Creds::new(&username, &password, &server) {
        match http::get_user(&creds) {
            Ok(_) => match keyring::set_creds("username", &creds) {
                Ok(_) => println!("Login Successful"),
                Err(_) => util::exit_failure("Failed to save credentials"),
            },
            Err(e) => util::exit_failure(&e.to_string()),
        }
    } else {
        util::exit_failure("Invalid url");
    };
}

/// Logout of the nextcloud server
fn logout() {
    match keyring::delete_creds("username") {
        Ok(_) => println!("Logout Successful"),
        Err(_) => println!("Error: Faild to logout"),
    }
}

/// Prints the username and server of logged in user
fn status() {
    let user = keyring::get_creds("username");
    match user {
        Ok(res) => {
            let username: String = res.username;
            let server: Url = res.server;
            println!("Logged in as {} for server {}", username, server);
        }
        Err(_) => println!("Not logged in"),
    }
}

/// Pulls a file from the server to your computer
fn pull(source: PathBuf, destination: PathBuf) {
    if let Ok(i) = keyring::get_creds("username") {
        let data: Bytes = http::get_file(&i, &source).unwrap();
        let file_name = source.file_name().unwrap_or(OsStr::new(""));

        let new_file_path = if destination.file_name().is_none() {
            Path::new("").join(&destination).join(file_name)
        } else {
            destination
        };

        file::create_file(&new_file_path, &data).unwrap();
        println!("Pulled {:?}, {:?}", source, new_file_path);
    }
}

/// Pushes a file from your computer to the server
fn push(source: PathBuf, destination: PathBuf) {
    if let Ok(i) = keyring::get_creds("username") {
        let data: Bytes = file::read_file(&source).unwrap();
        let file_name = source.file_name().unwrap_or(OsStr::new(""));

        let new_file_path = if destination.file_name().is_none() {
            Path::new("").join(&destination).join(file_name)
        } else {
            destination
        };

        http::send_file(&i, &new_file_path, data).unwrap();
        println!("Push {:?}, {:?}", source, new_file_path)
    }
}
