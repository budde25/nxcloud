use std::collections::HashMap;
use url::Url;

#[tokio::main]
pub async fn getAppPassword(server: Url, username: String, password: String) {
    let resp = reqwest::get(server + "/ocs/v2.php/core/getapppassword")
        .send()
        .await?;
    println!("{:#?}", resp);
}

#[tokio::main]
pub async fn login() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("{:#?}", resp);
    Ok(())
}
