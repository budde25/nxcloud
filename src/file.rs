use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, fs::File};

use anyhow::{anyhow, Result};
use base64::{decode, encode};
use bytes::Bytes;
use dirs_next::home_dir;
use once_cell::unsync::Lazy;

use super::Credentials;

pub const HISTORY_PATH: Lazy<PathBuf> = Lazy::new(|| {
    home_dir().unwrap().join(".cache/nxcloud_history.txt")
});

const CREDENTIALS_PATH: Lazy<PathBuf> = Lazy::new(|| {
    home_dir().unwrap().join(".cache/nxcloud_auth.txt")
});

impl Credentials {
    pub fn file_read_default() -> Result<Self> {
        Self::file_read(&CREDENTIALS_PATH)
    }

    fn file_read(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let contents_decoded = decode(contents)?;
        let decoded = String::from_utf8_lossy(&contents_decoded);
        let v: Vec<&str> = decoded.split(' ').collect();

        if v.len() != 3 {
            return Err(anyhow!("Unexpected format"));
        }

        Ok(Self::from(v[0], v[1], v[2])?)
    }

    pub fn file_write_default(&self) -> Result<()> {
        Self::file_write(&self, &CREDENTIALS_PATH)
    }

    fn file_write(&self, path: &Path) -> Result<()> {
        file_delete(path)?;

        let encoded = encode(self.display());
        let mut file = File::create(&path)?;
        file.write_all(encoded.as_bytes())?;
        Ok(())
    }

    pub fn file_delete_default() -> Result<()> {
        file_delete(&CREDENTIALS_PATH)
    }
}

pub fn file_delete(path: &Path) -> Result<()> {
    if path.exists() && path.is_file() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn create_file(path: &Path, data: &Bytes) -> Result<()> {
    if !path.exists() && !path.is_dir() {
        let mut file = File::create(&path)?;
        file.write_all(data)?;
    }
    Ok(())
}

pub fn read_file(path: &Path) -> Result<Bytes> {
    let contents = fs::read_to_string(path)?;
    Ok(Bytes::from(contents))
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn read_user_no_file() {
        let path = Path::new("test_user_no_file.txt");
        file_delete(path).unwrap();
        Credentials::file_read(path).expect_err("File should not exist");
    }

    #[test]
    fn write_user_no_file() {
        let path = Path::new("test_write_user_no_file.txt");
        let creds =
            Credentials::from("user", "pass", "https://cloud.example.com")
                .unwrap();
        creds.file_write(path).expect("File should be created");
        file_delete(path).unwrap();
    }

    #[test]
    fn write_user_overwrite_file() {
        let path = Path::new("test_write_user_overwrite_file.txt");
        let creds =
            Credentials::from("user", "pass", "https://cloud.example.com")
                .unwrap();
        creds.file_write(path).expect("File should be created");
        let creds2 =
            Credentials::from("user2", "pass2", "cloud.example2.com").unwrap();

        // https should be added dynamically
        creds2.file_write(path).expect("File should be created");
        let resp = Credentials::file_read(path).unwrap();
        assert_eq!(resp.username, String::from("user2"));
        assert_eq!(resp.password, String::from("pass2"));
        assert_eq!(
            resp.server,
            Url::parse("https://cloud.example2.com").unwrap()
        );
        file_delete(path).unwrap();
    }

    #[test]
    fn write_and_read() {
        let path = Path::new("test_read_and_write.txt");
        let creds =
            Credentials::from("user", "pass", "https://cloud.example.com")
                .unwrap();
        creds.file_write(path).expect("File should be created");
        let resp = Credentials::file_read(path).unwrap();
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
        file_delete(path).unwrap();
    }
}
