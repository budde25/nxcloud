use super::Credentials;
use color_eyre::eyre::WrapErr;
use color_eyre::Result;

#[cfg(feature = "secure-password")]
use keyring::Entry;

#[cfg(feature = "secure-password")]
const SERVICE_NAME: &str = "nextcloud_client_cli";

impl Credentials {
    #[cfg(feature = "secure-password")]
    pub fn write(&self) -> Result<()> {
        let keyring = Entry::new(SERVICE_NAME, "username");
        let encoded = self.encode();
        keyring
            .set_password(&encoded)
            .wrap_err("Failed to insert credentials into keyring")?;
        Ok(())
    }

    #[cfg(not(feature = "secure-password"))]
    pub fn write(&self) -> Result<()> {
        self.file_write_default()?;
        Ok(())
    }

    #[cfg(feature = "secure-password")]
    pub fn read() -> Result<Self> {
        let entry = Entry::new(SERVICE_NAME, "username");
        let content = entry
            .get_password()
            .wrap_err("Failed to remove credentials from keyring")?;
        Credentials::decode(&content)
    }

    #[cfg(not(feature = "secure-password"))]
    pub fn read() -> Result<Self> {
        Credentials::file_read_default()
    }

    #[cfg(feature = "secure-password")]
    pub fn delete() -> Result<()> {
        if cfg!(feature = "secure-password") {
            let entry = Entry::new(SERVICE_NAME, "username");
            if entry.delete_password().is_err() {
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
    use crate::{Credentials, Password, Server, Username};
    use url::Url;

    #[test]
    #[ignore]
    fn store_creds() {
        let creds = Credentials::parse(
            "test",
            "KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS",
            "https://cloud.example.com",
        );
        assert!(creds.is_ok());
        creds.unwrap().write().expect("Write should be possible");
        Credentials::delete().expect("Should remove creds");
    }

    #[test]
    #[ignore]
    fn set_and_read_creds() {
        let creds =
            Credentials::parse("test", "pass", "https://cloud.example.com");
        assert!(creds.is_ok());
        let creds = creds.unwrap();
        creds.write().expect("Args are valid should return a result");
        let creds = Credentials::read().expect("Should be creds");
        assert_eq!(creds.username, Username::new("test".to_string()));
        assert_eq!(creds.password, Password::new("pass".to_string()));
        assert_eq!(
            creds.server,
            Server::new(Url::parse("https://cloud.example.com").unwrap())
                .unwrap(),
        );
        assert_ne!(creds.username, Username::new("user2".to_string()));
    }
}
