use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Error;
use std::time::Duration;
use url::Url;

fn get_client() -> Result<Client, Error> {
    let timeout = Duration::new(10, 0);
    return Ok(ClientBuilder::new().timeout(timeout).build()?);
}

#[tokio::main]
pub async fn get_user(server: Url, user: &str, pass: &str) -> Result<String, Error> {
    let request = format!(
        "{url}{ext}{user}",
        url = server.as_str(),
        ext = "ocs/v1.php/cloud/users/",
        user = user
    );
    println!("request url: {}", request);

    let client = get_client().unwrap();

    let response = client
        .get(&request)
        .basic_auth(user, Some(pass))
        .header("OCS-APIRequest", "true")
        .send()
        .await?;

    let text = response.text().await?;
    return Ok(text);
}
