use super::Creds;
use anyhow::anyhow;
use base64::{decode, encode};
use keyring::Keyring;
use std::error::Error;
use url::Url;

const SERVICE_NAME: &str = "nextcloud_client_cli";

pub fn set_creds(username: &str, creds: &Creds) -> Result<(), anyhow::Error> {
    let keyring = Keyring::new(SERVICE_NAME, username);
    let creds_string = format!("{} {} {}", creds.username, creds.password, creds.server);
    let content = encode(creds_string);
    if let Err(_) = keyring.set_password(&content) {
        return Err(anyhow!("Keyring failed to set password"));
    }
    Ok(())
}

pub fn get_creds(username: &str) -> Result<Creds, anyhow::Error> {
    let keyring = Keyring::new(SERVICE_NAME, username);
    if let Ok(content) = keyring.get_password() {
        let data = String::from_utf8_lossy(&decode(content)?).to_string();

        let v: Vec<&str> = data.split(' ').collect();

        Ok(Creds {
            username: String::from(v[0]),
            password: String::from(v[1]),
            server: Url::parse(v[2])?,
        })
    } else {
        Err(anyhow!("Keyring failed to retrive password"))
    }
}

pub fn delete_creds(username: &str) -> Result<(), Box<dyn Error>> {
    let keyring = Keyring::new(SERVICE_NAME, username);
    keyring.delete_password()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn store_creds() {
        let url = Url::parse("https://cloud.example.com").unwrap();
        set_creds(
            "test",
            &Creds {
                server: url,
                username: String::from("test"),
                password: String::from("KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS"),
            },
        )
        .expect("Args are valid should return a result");
        delete_creds("test").expect("Should remove creds");
    }

    #[test]
    #[ignore]
    fn set_and_read_creds() {
        let url = Url::parse("https://cloud.example.com").unwrap();
        set_creds(
            "test1",
            &Creds {
                server: url,
                username: String::from("test"),
                password: String::from("pass"),
            },
        )
        .expect("Args are valid should return a result");
        let creds = get_creds("test1").expect("Should be creds");
        assert_eq!(creds.username, String::from("test"));
        assert_eq!(creds.password, String::from("pass"));
        assert_eq!(
            creds.server,
            Url::parse("https://cloud.example.com").unwrap()
        );
        assert_ne!(creds.username, String::from("user2"));
    }
}
