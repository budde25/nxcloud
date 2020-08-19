use base64::{decode, encode};
use bytes::Bytes;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use url::Url;

pub const DEFAULT_PATH: &str = "config.txt";

#[derive(Debug)]
pub struct Creds {
    pub username: String,
    pub password: String,
    pub server: Url,
}

pub fn read_user(path: &Path) -> Result<Creds, String> {
    let contents = fs::read_to_string(path);
    let res = match contents {
        Ok(i) => i,
        Err(_) => return Err(String::from("Failed to read file")),
    };
    let decoded = match &decode(res) {
        Ok(i) => String::from_utf8_lossy(i).to_string(),
        Err(_) => return Err(String::from("Failed to decode file")),
    };
    let v: Vec<&str> = decoded.split(' ').collect();

    if v.len() != 3 {
        return Err(String::from("Unexpect format"));
    }
    let creds: Creds = Creds {
        username: String::from(v[0]),
        password: String::from(v[1]),
        server: Url::parse(v[2]).unwrap(),
    };

    return Ok(creds);
}

pub fn write_user(creds: Creds, path: &Path) -> Result<(), io::Error> {
    remove_file(path);

    let contents = format!("{} {} {}", creds.username, creds.password, creds.server);
    let encoded = encode(contents);
    let mut file = File::create(&path)?;
    file.write(encoded.as_bytes())?;
    return Ok(());
}

pub fn remove_file(path: &Path) -> bool {
    if path.exists() && path.is_file() {
        fs::remove_file(path).expect("Error: Failed remove to file");
        return true;
    } else {
        return false;
    }
}

pub fn create_file(path: &Path, data: &Bytes) -> Result<(), io::Error> {
    if !path.exists() && !path.is_dir() {
        let mut file = File::create(&path)?;
        file.write(data)?;
    }
    return Ok(());
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_user_no_file() {
        let path = Path::new("test_user_no_file.txt");
        remove_file(path);
        read_user(path).expect_err("File should not exist");
    }

    #[test]
    fn write_user_no_file() {
        let path = Path::new("test_write_user_no_file.txt");
        write_user(
            Creds {
                username: String::from("user"),
                password: String::from("pass"),
                server: Url::parse("https://cloud.example.com").unwrap(),
            },
            path,
        )
        .expect("File should be created");
        assert!(remove_file(path));
    }

    #[test]
    fn write_user_overwrite_file() {
        let path = Path::new("test_write_user_overwrite_file.txt");
        write_user(
            Creds {
                username: String::from("user"),
                password: String::from("pass"),
                server: Url::parse("https://cloud.example.com").unwrap(),
            },
            path,
        )
        .expect("File should be created");
        write_user(
            Creds {
                username: String::from("user2"),
                password: String::from("pass2"),
                server: Url::parse("https://cloud.example2.com").unwrap(),
            },
            path,
        )
        .expect("File should be created");
        let resp = read_user(path).unwrap();
        assert_eq!(resp.username, String::from("user2"));
        assert_eq!(resp.password, String::from("pass2"));
        assert_eq!(
            resp.server,
            Url::parse("https://cloud.example2.com").unwrap()
        );
        assert!(remove_file(path));
    }

    #[test]
    fn write_and_read() {
        let path = Path::new("test_read_and_write.txt");
        remove_file(path);
        write_user(
            Creds {
                username: String::from("user"),
                password: String::from("pass"),
                server: Url::parse("https://cloud.example.com").unwrap(),
            },
            path,
        )
        .expect("File should be created");
        let resp = read_user(path).unwrap();
        assert_eq!(resp.username, String::from("user"));
        assert_eq!(resp.password, String::from("pass"));
        assert_eq!(
            resp.server,
            Url::parse("https://cloud.example.com").unwrap()
        );

        assert_ne!(resp.username, String::from("user2"));
        assert_ne!(resp.password, String::from("pass2"));
        assert_ne!(
            resp.server,
            Url::parse("https://cloud.example2.com").unwrap()
        );
        assert!(remove_file(path));
    }
}
