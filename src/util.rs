use std::process::exit;

// Exits the program while printing the error
pub fn exit_failure(error: &str) {
    eprintln!("Error: {}", error);
    exit(1);
}
