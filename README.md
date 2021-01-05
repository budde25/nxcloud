# NXCloud - NextCloud Client CLI

[![Build Status](https://travis-ci.com/budde25/nextcloud-client-cli.svg?token=h2VpZBHUrSqEk7SD3Acq&branch=master)](https://travis-ci.com/budde25/nextcloud-client-cli)
[![Crates.io](https://img.shields.io/crates/v/nxcloud)](https://crates.io/crates/nxcloud)
[![Crates.io](https://img.shields.io/crates/d/nxcloud)](https://crates.io/crates/nxcloud)
[![nxcloud](https://snapcraft.io//nxcloud/badge.svg)](https://snapcraft.io/nxcloud)

A CLI client to interact with a NextCloud server. It makes it possible to push and pull files between a client and a NextCloud server without the need of a GUI. This CLI is completly compatatible with Linux. It has also been built with Windows and MacOs in mind, but they are untested ATM and mileage may vary.

## Install

`snap install nxcloud` reccommended.  
`cargo install nxcloud`, requires extra packages to compile (list in setup), not reccomended.

## Usage

```none
NxCloud 0.2.0
A command line client for interacting with your NextCloud server.

USAGE:
    nxcloud [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode (-v, -vv, -vvv)

SUBCOMMANDS:
    cd        Change directory of remote - Shell Only
    help      Prints this message or the help of the given subcommand(s)
    login     Login to your NextCloud server, please provide a app password for security
    logout    Logout of your NextCloud server
    ls        List files and directories
    mkdir     Make a directory
    pull      Pull a file from the server to your local machine
    push      Push a file from your local machine to the server
    rm        Remove a file or directory, WARNING deletes files recursively
    shell     Enter an interactive prompt
    status    Display's the account status
```

Use `nxcloud <subcommand> help` for help with that subcommand.  

## Setup

* Install [Rust](https://www.rust-lang.org/tools/install)  
* Install libdbus-1-dev, build-essential, libssl-dev (Linux only)  
`apt install libdbus-1-dev build-essential libssl-dev` (Debain based)
* Clone repository  

### Compile and Run

`cargo build` Will build an executable.  
`cargo run -- <args>` Will build and run an executable.  
`cargo doc` Will build the documentation.  

### Testing

`cargo test` Will run all the unit tests except for the ignored ones, ignored because they use network and won't pass 100% reliably.  
`cargo test -- --ignored` Will run all the tests, some may fail depending on server response time and your internet capabilities.  

## Built With

[Rust](https://www.rust-lang.org/)

## License

[GNU General Public License v3.0](https://github.com/budde25/nextcloud-client-cli/blob/master/LICENSE)  

## Author

Ethan Budd
