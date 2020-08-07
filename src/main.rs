use std::path::PathBuf;
use structopt::StructOpt;
use url::Url;

mod http;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "nextcloudcli",
    about = "A cli client for interacting with your NextCloud server."
)]
enum Cli {
    /// Display's the account status.
    #[structopt(name = "status")]
    Status {},
    #[structopt(name = "login")]
    /// Login to your NextCloud server, please provide a app password for sucurity.
    Login {
        /// The server url, ex: https://nextcloud.com.
        #[structopt(short = "s", long)]
        server: Url,
        /// Your NextCloud username.
        #[structopt(short = "u", long)]
        username: String,
        /// A NextCloud app password, do not use your account password.
        #[structopt(short = "p", long)]
        password: String,
    },
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
        Cli::Status {} => println!("Status"),
        Cli::Login {
            server,
            username,
            password,
        } => login(server, &username, &password),
        Cli::Push {
            source,
            destination,
        } => println!("Push"),
        Cli::Pull {
            source,
            destination,
        } => println!("Pull"),
    };
}

fn login(server: Url, username: &str, password: &str) {
    let res = http::get_user(server, &username, &password);
    match res {
        Ok(i) => println!("{:?}", i),
        Err(i) => println!("{:?}", i),
    }
}
