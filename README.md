<!-- Title -->
# NXCloud

<!-- Subtitle-->
NextCloud Client Command Line Interface

<!-- Shields -->
<!-- TODO add GitHub actions -->
[![Crates.io](https://flat.badgen.net/crates/v/nxcloud)](https://crates.io/crates/nxcloud)
[![Crates.io](https://flat.badgen.net/crates/d/nxcloud)](https://crates.io/crates/nxcloud)
[![License](https://flat.badgen.net/badge/license/MIT/blue)](LICENSE-MIT)
[![License](https://flat.badgen.net/badge/license/APACHE/blue)](LICENSE-APACHE)


<!-- Table of Contents -->
<details open="false">
  <summary><strong>Table of Contents</strong></summary>
  <ol>
    <li><a href="#about">About</a></li>
    <li><a href="#installation">Installation</a></li>
    <li>
      <a href="#usage">Usage</a>
      <ul>
        <li><a href="#examples">Examples</a></li>
      </ul>
    </li>
    <li>
      <a href="#building-and-testing">Building and Testing</a>
        <ul>
          <li><a href="#setup">Setup</a></li>
          <li><a href="#compile-and-run">Compile and Run</a></li>
          <li><a href="#testing">Testing</a></li>
          <li><a href="#documentation">Documentation</a></li>
        </ul>
    </li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#status">Status</a></li>
    <li><a href="#built-with">Built With</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#license">License</a></li>
  </ol>
</details>

<!-- Info -->
## About

<!-- Image/GIF -->

A CLI client to interact with a NextCloud server.  

Features:  
* Allows for pushing and pulling files on the server.
* Creating and Deleting remote directories.
* Listing files.
* Interactive browsing through the shell command

This application makes it possible to exchange a client and a NextCloud server without the need of a GUI. This CLI is compatible with Linux. It has also been built with Windows and MacOs in mind, they are untested at the moment and mileage may vary.

<!-- Installation -->
## Installation

For a general cross platform linux a [Snap package](https://snapcraft.io/nxcloud) is available.  
`snap install nxcloud`  
  
If you have rust installed you can use cargo.  
requires a the following packages to be install:
libdbus-1-dev, build-essential, libssl-dev (Debian names ,probably installed by default)  
`cargo install nxcloud`   
  
Other packaged binary's are available in [Releases](https://github.com/budde25/nextcloud-client-cli/releases)  

<!-- Usage -->
## Usage

The binary name is `nxcloud`  

To display application use `nxcloud help`  
Use `nxcloud <subcommand> help` for help with that subcommand.  

<!-- Examples -->
### Examples
To start using the interacting with you're NextCloud you need to login.  
Use an app password as opposed your account password.  
`nxcloud login <server> <username> <password>`

Listing files in a directory, support -l and -a.  
`nxcloud ls -la`

Pushing and pulling is very simple.  
`nxcloud pull <source file path (remote)> <destination file path (local)>`  
`nxcloud pull <source file path (local)> <destination file path (remote)>`  

Entering a shell to remember current (remote) directory (Experimental).  
Allows usage of the cd subcommand.  
`nxcloud shell`  

<!-- Building and Testing -->
## Building and Testing

This repository is a standard rust project bin structure.  

<!-- Setup -->
### Setup

* Install [Rust](https://www.rust-lang.org/tools/install)  
* Install libdbus-1-dev, build-essential, libssl-dev (Linux) <br> `apt install libdbus-1-dev build-essential libssl-dev` (Debian based)
* Clone repository

<!-- Compile and Run -->
### Compile and Run

Rust support building or running with the following commands:  
`cargo build` Will build an executable in `/target/debug/`.  
`cargo run -- <args>` Will build and run an executable.    

<!-- Testing -->
### Testing

Testing all standard test can be done with rust built in test framework.  
`cargo test`

Some tests cannot be completed with 100% reliability (for example they might fail without network access), this will run all ignored tests.  
`cargo test -- --ignored`

<!-- Docs -->
### Documentation

Rust built in documentation tools can be generated.  
`cargo doc`

To open with your default browser.  
`cargo doc --open`

<!-- Contributing -->
## Contributing

Contributions are completely welcome and encouraged!  
Examples of contributing could include: 

* Submitting a feature request or bug report.  
* Asking for improved documentation.  
* Code by creating a pull request.  

Refer to [Contributing](CONTRIBUTING.md)

<!-- Development Status -->
## Status

Development is still in progress with new features being planned.  
Feel free to [Contribute](#Contributing).

<!-- Technologies -->
## Built With

[Rust](https://www.rust-lang.org/)

<!-- Contact Info -->
## Contact

Created by [Ethan Budd](https://github.com/budde25)  
Email: [budde25@protonmail.com](mailto:budde25@protonmail.com)  

<!-- License -->
## License

Dual-licensed under either either of the following:
* [MIT License](LICENSE-MIT)
* [Apache License](LICENSE-APACHE)
