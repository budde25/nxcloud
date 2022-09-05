use color_eyre::eyre::Context;
use color_eyre::Result;
use dirs_next::cache_dir;
use once_cell::sync::Lazy;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, fs::File};

use super::Credentials;

pub static HISTORY_PATH: Lazy<PathBuf> =
    Lazy::new(|| cache_dir().unwrap().join("nxcloud_history.txt"));

static CREDENTIALS_PATH: Lazy<PathBuf> =
    Lazy::new(|| cache_dir().unwrap().join(".nxcloud_auth.txt"));

impl Credentials {
    pub fn read_default() -> Result<Self> {
        Self::parse_file(&CREDENTIALS_PATH)
    }

    pub fn file_write_default(&self) -> Result<()> {
        Self::file_write(self, &CREDENTIALS_PATH)
    }

    fn file_write(&self, path: &Path) -> Result<()> {
        file_delete(path)?;
        let encoded = self.encode();
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
        fs::remove_file(path).wrap_err_with(|| {
            format!("Failed to delete file from {}", path.display())
        })?;
    }
    Ok(())
}

pub fn create_file(path: &Path, data: &[u8]) -> Result<()> {
    if !path.exists() && !path.is_dir() {
        let mut file = File::create(&path).wrap_err_with(|| {
            format!("Failed to write file from {}", path.display())
        })?;
        file.write_all(data)?;
    }
    Ok(())
}

// TESTS
#[cfg(test)]
mod tests {
    use crate::types::credentials::{Password, Server, Username};

    use super::*;
    use tempfile;
    use url::Url;

    #[test]
    fn read_user_no_file() {
        let path = Path::new("test_user_no_file.txt");
        file_delete(path).unwrap();
        fs::read(path).expect_err("File should not exist");
    }

    #[test]
    fn write_user_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir
            .path()
            .to_path_buf()
            .join(Path::new("test_write_user_no_file.txt"));
        let creds =
            Credentials::parse("user", "pass", "https://cloud.example.com")
                .unwrap();
        creds.file_write(&path).expect("File should be created");
        file_delete(&path).unwrap();
    }

    #[test]
    fn write_user_overwrite_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir
            .path()
            .to_path_buf()
            .join(Path::new("test_write_user_overwrite_file.txt"));
        let creds =
            Credentials::parse("user", "pass", "https://cloud.example.com")
                .unwrap();
        creds.file_write(&path).expect("File should be created");
        let creds2 =
            Credentials::parse("user2", "pass2", "https://cloud.example2.com")
                .unwrap();

        // https should be added dynamically
        creds2.file_write(&path).expect("File should be created");
        let resp = Credentials::parse_file(&path).unwrap();
        assert_eq!(resp.username, Username::new("user2".to_string()));
        assert_eq!(resp.password, Password::new("pass2".to_string()));
        assert_eq!(
            resp.server,
            Server::new(Url::parse("https://cloud.example2.com").unwrap())
                .unwrap()
        );
        file_delete(&path).unwrap();
    }

    #[test]
    fn write_and_read() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir
            .path()
            .to_path_buf()
            .join(Path::new("test_read_and_write.txt"));
        let creds =
            Credentials::parse("user", "pass", "https://cloud.example.com")
                .unwrap();
        creds.file_write(&path).expect("File should be created");
        let resp = Credentials::parse_file(&path).unwrap();
        assert_eq!(resp.username, Username::new("user".to_string()));
        assert_eq!(resp.password, Password::new("pass".to_string()));
        assert_eq!(
            resp.server,
            Server::new(Url::parse("https://cloud.example.com").unwrap())
                .unwrap()
        );

        assert_ne!(resp.username, Username::new("user2".to_string()));
        assert_ne!(resp.password, Password::new("pass2".to_string()));
        assert_ne!(
            resp.server,
            Server::new(Url::parse("https://cloud.example2.com").unwrap())
                .unwrap()
        );
        file_delete(&path).unwrap();
    }
}
