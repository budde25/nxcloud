use super::Credentials;
use anyhow::Result;

#[cfg(feature = "secure-password")]
use base64::{decode, encode};
#[cfg(feature = "secure-password")]
use keyring::Keyring;

#[cfg(feature = "secure-password")]
const SERVICE_NAME: &str = "nextcloud_client_cli";

impl Credentials {
    #[cfg(feature = "secure-password")]
    pub fn write(&self) -> Result<()> {
        let keyring = Keyring::new(SERVICE_NAME, "username");
        let credentials_string =
            format!("{} {} {}", self.username, self.password, self.server);
        let content = encode(credentials_string);
        if keyring.set_password(&content).is_err() {
            self.file_write_default()?;
        }
        Ok(())
    }

    #[cfg(not(feature = "secure-password"))]
    pub fn write(&self) -> Result<()> {
        self.file_write_default()?;
        Ok(())
    }

    #[cfg(feature = "secure-password")]
    pub fn read() -> Result<Self> {
        let keyring = Keyring::new(SERVICE_NAME, "username");
        if let Ok(content) = keyring.get_password() {
            let data = String::from_utf8_lossy(&decode(content)?).to_string();

            let v: Vec<&str> = data.split(' ').collect();

            Ok(Self::from(v[0], v[1], v[2])?)
        } else {
            Credentials::file_read_default()
        }
    }

    #[cfg(not(feature = "secure-password"))]
    pub fn read() -> Result<Self> {
        Credentials::file_read_default()
    }

    #[cfg(feature = "secure-password")]
    pub fn delete() -> Result<()> {
        if cfg!(feature = "secure-password") {
            let keyring = Keyring::new(SERVICE_NAME, "username");
            if keyring.delete_password().is_err() {
                Credentials::file_delete_default()?;
            }
        } else {
            Credentials::file_delete_default()?;
        }
        Ok(())
    }

    #[cfg(not(feature = "secure-password"))]
    pub fn delete() -> Result<()> {
        Credentials::file_delete_default()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    #[ignore]
    fn store_creds() {
        let url = Url::parse("https://cloud.example.com").unwrap();
        let creds =
            Credentials::new("test", "KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS", url);
        creds.write().expect("Write should be possible");
        Credentials::delete().expect("Should remove creds");
    }

    #[test]
    #[ignore]
    fn set_and_read_creds() {
        let url = Url::parse("https://cloud.example.com").unwrap();
        let creds = Credentials::new("test", "pass", url);
        creds.write().expect("Args are valid should return a result");
        let creds = Credentials::read().expect("Should be creds");
        assert_eq!(creds.username, String::from("test"));
        assert_eq!(creds.password, String::from("pass"));
        assert_eq!(
            creds.server,
            Url::parse("https://cloud.example.com").unwrap()
        );
        assert_ne!(creds.username, String::from("user2"));
    }
}
