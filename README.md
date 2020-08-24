# NextCloud Client CLI

[![Build Status](https://travis-ci.com/budde25/nextcloud-client-cli.svg?token=h2VpZBHUrSqEk7SD3Acq&branch=master)](https://travis-ci.com/budde25/nextcloud-client-cli)

A CLI client to interact with a NextCloud server. It makes it possible to push and pull files between a client and a NextCloud server without the need of a GUI. This CLI is completly compatatible with Linux. It has also been built with Windows and MacOs in mind, but they are untested ATM and mileage may vary.

## Usage

`nxcloud -h` Show help.  
`nxcloud login -s <server url> -u <username> -p <app password` Login to NextCloud server. Note, please make an app password from the security tab, DO NOT use you regular password.  
`nxcloud logout` Logout of NextCloud server.  
`nxcloud status` Displays whether a user is currently logged in, and to which NextCloud server.
`nxcloud push <source> <destination>` Push a file from local machine to NextCloud server.  
`nxcloud pull <source> <destination>` Pull a file from NextCloud server to local machine.  
`nxcloud -V` Display version info.  

## Setup
* Install [Rust](https://www.rust-lang.org/tools/install).  
* Clone repostitory.  

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
