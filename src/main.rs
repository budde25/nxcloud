use bytes::Bytes;
use file::DEFAULT_PATH;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;
use url::Url;

mod file;
mod http;
mod keyring;

#[derive(Debug)]
pub struct Creds {
    pub username: String,
    pub password: String,
    pub server: Url,
}

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
    exit(0);
}

fn login(server: String, username: String, password: String) {
    let fqdn: String = if !server.contains("https://") || !server.contains("http://") {
        format!("https://{}", &server)
    } else {
        server
    };

    if let Ok(res) = Url::parse(&fqdn) {
        let url = res;

        let creds = Creds {
            username,
            password,
            server: url,
        };
        let resp = http::get_user(&creds);
        match resp {
            Ok(_) => {
                match keyring::set_creds("username", &creds) {
                    Ok(_) => println!("Login Successful"),
                    Err(_) => println!("Error: Faild to save credentials"),
                }
                return;
            }
            Err(e) => exit_failure(&e.to_string()),
        }
    } else {
        exit_failure("Invalid url");
    }
}

fn logout() {
    match keyring::delete_creds("username") {
        Ok(_) => println!("Logout Successful"),
        Err(_) => println!("Error: Faild to logout"),
    }
}

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

fn exit_failure(error: &str) {
    eprintln!("Error: {}", error);
    exit(1);
}
