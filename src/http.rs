use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;
use reqwest::{Client, ClientBuilder, Method};

use super::Credentials;

pub struct Http {
    credentials: Credentials,
    client: Client,
}

impl Credentials {
    pub fn into_http(self) -> Http {
        Http::from(self)
    }
}

impl Http {
    pub fn from(credentials: Credentials) -> Self {
        Self {
            credentials,
            client: ClientBuilder::new()
                .timeout(Duration::new(10, 0))
                .build()
                .unwrap(),
        }
    }

    #[tokio::main]
    pub async fn get_user(&self) -> Result<String> {
        let request: String = format!(
            "{url}{ext}{user}",
            url = self.credentials.server,
            ext = "ocs/v1.php/cloud/users/",
            user = self.credentials.username
        );

        let response = self
            .client
            .get(&request)
            .basic_auth(
                &self.credentials.username,
                Some(&self.credentials.password),
            )
            .header("OCS-APIRequest", "true")
            .send()
            .await?
            .error_for_status();

        Ok(response?.text().await?)
    }

    #[tokio::main]
    pub async fn get_file(&self, path: &Path) -> Result<Bytes> {
        let request: String = format!(
            "{url}{ext}{user}/{path}",
            url = self.credentials.server,
            ext = "remote.php/dav/files/",
            user = self.credentials.username,
            path = path.to_string_lossy()
        );

        let response = self
            .client
            .get(&request)
            .basic_auth(
                &self.credentials.username,
                Some(&self.credentials.password),
            )
            .send()
            .await?
            .error_for_status();

        Ok(response?.bytes().await?)
    }

    #[tokio::main]
    pub async fn send_file(self, path: &Path, data: Bytes) -> Result<()> {
        let request: String = format!(
            "{url}{ext}{user}/{path}",
            url = self.credentials.server,
            ext = "remote.php/dav/files/",
            user = self.credentials.username,
            path = path.to_string_lossy()
        );

        self.client
            .put(&request)
            .basic_auth(
                self.credentials.username,
                Some(self.credentials.password),
            )
            .header("OCS-APIRequest", "true")
            .body(data)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tokio::main]
    pub async fn make_folder(self, path: &Path) -> Result<()> {
        let request: String = format!(
            "{url}{ext}{user}/{path}",
            url = self.credentials.server,
            ext = "remote.php/dav/files/",
            user = self.credentials.username,
            path = path.to_string_lossy()
        );

        self.client
            .request(Method::from_bytes(b"MKCOL").unwrap(), &request)
            .basic_auth(
                self.credentials.username,
                Some(self.credentials.password),
            )
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tokio::main]
    pub async fn delete(self, path: &Path) -> Result<()> {
        let request: String = format!(
            "{url}{ext}{user}/{path}",
            url = self.credentials.server,
            ext = "remote.php/dav/files/",
            user = self.credentials.username,
            path = path.to_string_lossy()
        );

        self.client
            .request(Method::from_bytes(b"DELETE").unwrap(), &request)
            .basic_auth(
                self.credentials.username,
                Some(self.credentials.password),
            )
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
    #[tokio::main]
    pub async fn get_list(self, path: &Path) -> Result<String> {
        let request: String = format!(
            "{url}{ext}{user}/{path}",
            url = self.credentials.server,
            ext = "remote.php/dav/files/",
            user = self.credentials.username,
            path = path.to_string_lossy()
        );

        const DATA: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <d:propfind xmlns:d=\"DAV:\">
      <d:prop xmlns:oc=\"http://owncloud.org/ns\">
        <d:getlastmodified/>
        <d:getcontentlength/>
        <d:getcontenttype/>
        <oc:permissions/>
        <d:resourcetype/>
        <d:getetag/>
      </d:prop>
    </d:propfind>";

        let response = self
            .client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &request)
            .basic_auth(
                self.credentials.username,
                Some(self.credentials.password),
            )
            .header("depth", "1")
            .body(DATA)
            .send()
            .await?
            .error_for_status();

        Ok(response?.text().await?)
    }
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn get_user_valid() {
        let http = Credentials::parse(
            "test",
            "KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS",
            "https://cloud.ebudd.io",
        )
        .unwrap()
        .into_http();
        http.get_user().expect("Args are valid should return a result");
    }

    #[test]
    fn get_user_invalid_url() {
        let http = Credentials::parse(
            "test",
            "KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS",
            "https://cloud.ebudd.i",
        )
        .unwrap()
        .into_http();
        http.get_user().expect_err("Url is invalid should fail");
    }

    #[test]
    #[ignore]
    fn get_user_invalid_creds() {
        let http = Credentials::parse(
            "test_wrong",
            "KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS",
            "https://cloud.ebudd.io",
        )
        .unwrap()
        .into_http();
        http.get_user().expect_err("Username is invalid should fail");
    }
}
