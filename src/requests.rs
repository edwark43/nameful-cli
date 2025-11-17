use reqwest::{self, blocking::Response, header::AUTHORIZATION};
use serde_json::Value;

pub fn api_get(path: &str) -> color_eyre::Result<Value> {
    let url = "https://newnameful.com/api/data";
    let body = reqwest::blocking::get(format!("{}{}", url, path))?.text()?;
    Ok(serde_json::from_str(&body)?)
}

pub fn api_put(path: &str, data: &str, key: &str) -> color_eyre::Result<Response> {
    let url = format!("https://newnameful.com/api/data{}", path);
    let client = reqwest::blocking::Client::new();
    let json: Value = serde_json::from_str(data)?;
    let request = client
        .put(url)
        .header(AUTHORIZATION, format!("Bearer {}", key))
        .json(&json)
        .send()?;
    Ok(request.error_for_status()?)
}

pub fn api_post(path: &str, data: &str, key: &str) -> color_eyre::Result<Response> {
    let url = format!("https://newnameful.com/api/data{}", path);
    let client = reqwest::blocking::Client::new();
    let json: Value = serde_json::from_str(data)?;
    let request = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", key))
        .json(&json)
        .send()?;
    Ok(request.error_for_status()?)
}

pub fn api_delete(path: &str, key: &str) -> color_eyre::Result<Response> {
    let url = format!("https://newnameful.com/api/data{}", path);
    let client = reqwest::blocking::Client::new();
    let request = client
        .delete(url)
        .header(AUTHORIZATION, format!("Bearer {}", key))
        .send()?;
    Ok(request.error_for_status()?)
}
