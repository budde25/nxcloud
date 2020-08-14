use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

const CONFIG: &str = "config.txt";

pub fn read_user() -> Result<(String, String), String> {
    let contents = fs::read_to_string(CONFIG);
    let res = match contents {
        Ok(i) => i,
        Err(_) => return Err(String::from("Failed to read file")),
    };
    let v: Vec<&str> = res.split(' ').collect();

    if v.len() != 2 {
        return Err(String::from("Unexpect format"));
    }
    let user = String::from(v[0]);
    let pass = String::from(v[1]);

    return Ok((user, pass));
}

pub fn write_user(user: &str, pass: &str) -> Result<(), io::Error> {
    let path = Path::new(CONFIG);
    if !path.exists() {
        fs::remove_file(path)?;
    }

    let contents = format!("{} {}", user, pass);
    let mut file = File::create(&path)?;
    file.write(contents.as_bytes())?;
    return Ok(());
}
