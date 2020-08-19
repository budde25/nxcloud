use super::file::Creds;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Error;
use reqwest::Response;
use std::time::Duration;
use xmltree::Element;

pub fn handle_response(result: Result<String, Error>) -> Result<Element, String> {
    match result {
        Ok(text) => match Element::parse(text.as_bytes()) {
            Ok(xml) => return Ok(xml),
            Err(e) => return Err(format!("Error: Failed to parse, {}, {}", e, text)),
        },
        Err(_) => return Err(String::from("Error: Failed to unwrap response data")),
    }
}

fn get_client() -> Result<Client, Error> {
    let timeout = Duration::new(10, 0);
    return Ok(ClientBuilder::new().timeout(timeout).build()?);
}

#[tokio::main]
pub async fn get_user(creds: &Creds) -> Result<String, Error> {
    let request: String = format!(
        "{url}{ext}{user}",
        url = creds.server.as_str(),
        ext = "ocs/v1.php/cloud/users/",
        user = creds.username
    );

    let client: Client = get_client()?;

    let response: Result<Response, Error> = client
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
        handle_response(resp).expect("Handle response should work");
    }
}
