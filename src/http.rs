extern crate xmltree;

use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Error;
use reqwest::Response;
use std::time::Duration;
use url::Url;
use xmltree::Element;

#[tokio::main]
pub async fn handle_response(result: Result<Response, Error>) -> Result<Element, String> {
    match result {
        Ok(resp) => match resp.text().await {
            Ok(text) => match Element::parse(text.as_bytes()) {
                Ok(xml) => return Ok(xml),
                Err(_) => return Err(String::from("Error: Failed to parse response")),
            },
            Err(_) => return Err(String::from("Error: Failed to unwrap response data")),
        },
        Err(err) => return Err(err.to_string()),
    }
}

fn get_client() -> Result<Client, Error> {
    let timeout = Duration::new(10, 0);
    return Ok(ClientBuilder::new().timeout(timeout).build()?);
}

#[tokio::main]
pub async fn get_user(server: Url, user: &str, pass: &str) -> Result<Response, Error> {
    let request: String = format!(
        "{url}{ext}{user}",
        url = server.as_str(),
        ext = "ocs/v1.php/cloud/users/",
        user = user
    );

    let client: Client = get_client()?;

    let response: Result<Response, Error> = client
        .get(&request)
        .basic_auth(user, Some(pass))
        .header("OCS-APIRequest", "true")
        .send()
        .await?
        .error_for_status();

    return response;
}
