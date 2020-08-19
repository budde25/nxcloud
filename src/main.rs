use bytes::Bytes;
use file::Creds;
use file::DEFAULT_PATH;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;
use url::Url;

mod file;
mod http;

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
                let path = Path::new(DEFAULT_PATH);
                match file::write_user(creds, path) {
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
    let path = Path::new(file::DEFAULT_PATH);
    file::remove_file(path);
    println!("Logout Successful");
}

fn status() {
    let path = Path::new(DEFAULT_PATH);
    let user = file::read_user(path);
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
    let path = Path::new(file::DEFAULT_PATH);
    if let Ok(i) = file::read_user(path) {
        let data: Bytes = http::get_file(&i, &source).unwrap();
        let file_stem = source.file_stem().unwrap_or(OsStr::new(""));
        let file_ext = source.extension().unwrap_or(OsStr::new(""));

        let file = format!(
            "{}.{}",
            file_stem.to_string_lossy(),
            file_ext.to_string_lossy()
        );
        let new_file_path = Path::new("").join(&destination).join(Path::new(&file));

        file::create_file(&new_file_path, &data).unwrap();
        println!(
            "Pull {:?}, {:?}, data: {:?}",
            source, destination, new_file_path
        );
    }
}

fn push(source: PathBuf, destination: PathBuf) {
    println!("Push {:?}, {:?}", source, destination)
}

fn exit_failure(error: &str) {
    eprintln!("Error: {}", error);
    exit(1);
}
