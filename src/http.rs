use super::Creds;
use bytes::Bytes;
use lazy_static::lazy_static;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Error;
use reqwest::Method;
use reqwest::Response;
use std::path::Path;
use std::time::Duration;

lazy_static! {
    static ref CLIENT: Client = {
        ClientBuilder::new()
            .timeout(Duration::new(10, 0))
            .build()
            .unwrap()
    };
}

#[tokio::main]
pub async fn get_user(creds: &Creds) -> Result<String, Error> {
    let request: String = format!(
        "{url}{ext}{user}",
        url = creds.server.as_str(),
        ext = "ocs/v1.php/cloud/users/",
        user = creds.username
    );

    let response: Result<Response, Error> = CLIENT
        .get(&request)
        .basic_auth(&creds.username, Some(&creds.password))
        .header("OCS-APIRequest", "true")
        .send()
        .await?
        .error_for_status();

    match response {
        Ok(resp) => return Ok(resp.text().await?),
        Err(e) => return Err(e),
    }
}

#[tokio::main]
pub async fn get_file(creds: &Creds, path: &Path) -> Result<Bytes, Error> {
    let request: String = format!(
        "{url}{ext}{user}/{path}",
        url = creds.server.as_str(),
        ext = "remote.php/dav/files/",
        user = creds.username,
        path = path.to_string_lossy()
    );

    let response: Result<Response, Error> = CLIENT
        .get(&request)
        .basic_auth(&creds.username, Some(&creds.password))
        .header("OCS-APIRequest", "true")
        .send()
        .await?
        .error_for_status();

    match response {
        Ok(resp) => return Ok(resp.bytes().await?),
        Err(e) => return Err(e),
    }
}

#[tokio::main]
pub async fn send_file(creds: &Creds, path: &Path, data: Bytes) -> Result<(), Error> {
    let request: String = format!(
        "{url}{ext}{user}/{path}",
        url = creds.server.as_str(),
        ext = "remote.php/dav/files/",
        user = creds.username,
        path = path.to_string_lossy()
    );

    let response: Result<Response, Error> = CLIENT
        .put(&request)
        .basic_auth(&creds.username, Some(&creds.password))
        .header("OCS-APIRequest", "true")
        .body(data)
        .send()
        .await?
        .error_for_status();

    match response {
        Ok(_) => return Ok(()),
        Err(e) => return Err(e),
    }
}

#[tokio::main]
pub async fn get_list(creds: &Creds, path: &Path) -> Result<String, Error> {
    let request: String = format!(
        "{url}{ext}{user}/{path}",
        url = creds.server.as_str(),
        ext = "remote.php/dav/files/",
        user = creds.username,
        path = path.to_string_lossy()
    );

    let data = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
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

    let response: Result<Response, Error> = CLIENT
        .request(Method::from_bytes(b"PROPFIND").unwrap(), &request)
        .basic_auth(&creds.username, Some(&creds.password))
        .header("depth", "1")
        .body(data)
        .send()
        .await?
        .error_for_status();

    match response {
        Ok(resp) => return Ok(resp.text().await?),
        Err(e) => return Err(e),
    }
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    #[ignore]
    fn get_user_valid() {
        let url = Url::parse("https://cloud.ebudd.io").unwrap();
        get_user(&Creds {
            server: url,
            username: String::from("test"),
            password: String::from("KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS"),
        })
        .expect("Args are valid should return a result");
    }

    #[test]
    fn get_user_invalid_url() {
        let url = Url::parse("https://cloud.ebudd.i").unwrap();
        get_user(&Creds {
            server: url,
            username: String::from("test"),
            password: String::from("KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS"),
        })
        .expect_err("Url is invalid should fail");
    }

    #[test]
    #[ignore]
    fn get_user_invalid_creds() {
        let url = Url::parse("https://cloud.ebudd.io").unwrap();
        get_user(&Creds {
            server: url,
            username: String::from("test_wrong"),
            password: String::from("KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS"),
        })
        .expect_err("Username is invalid should fail");
    }

    #[test]
    #[ignore]
    fn get_user_handle_response() {
        let url = Url::parse("https://cloud.ebudd.io").unwrap();
        let resp = get_user(&&Creds {
            server: url,
            username: String::from("test"),
            password: String::from("KXFJb-Pj8Ro-Rfkr4-q47CW-nwdWS"),
        });
    }
}
