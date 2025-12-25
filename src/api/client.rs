use futures::StreamExt;
use reqwest::{Body, Client, StatusCode};
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

pub enum Method {
    Post,
    Get,
}
pub async fn fetch_json<T, P>(
    url: &str,
    client: &Client,
    method: Method,
    body: Option<P>,
    headers: Option<Vec<(&str, &str)>>,
) -> Result<T, ClientError>
where
    T: DeserializeOwned,
    P: Into<Body>,
{
    let url = Url::parse(url)?;
    let request = match method {
        Method::Get => {
            let mut r = client.get(url);
            if let Some(headers) = headers {
                for (key, value) in headers {
                    r = r.header(key, value);
                }
            }
            r
        }
        Method::Post => {
            let mut r = client.post(url);
            if let Some(body) = body {
                r = r.body(body);
            }
            if let Some(headers) = headers {
                for (key, value) in headers {
                    println!("Header: {} = {}", key, value);
                    r = r.header(key, value);
                }
            }
            r
        }
    };

    let response = request.send().await?;
    let status = response.status();

    if !status.is_success() {
        let response_text = response.text().await?;
        return Err(ClientError::Http {
            status,
            body: response_text,
        });
    }
    let response_text = response.text().await?;
    let data = serde_json::from_str(&response_text)?;
    Ok(data)
}

pub async fn fetch_file<F>(url: &str, client: &Client, callback: F) -> Result<Vec<u8>, ClientError>
where
    F: Fn(i64),
{
    let url = Url::parse(url)?;
    let request = client.get(url);

    let response = request.send().await?;
    let status = response.status();

    if !status.is_success() {
        let response_text = response.text().await?;
        return Err(ClientError::Http {
            status,
            body: response_text,
        });
    }

    let mut stream = response.bytes_stream();

    let mut buffer: Vec<u8> = Vec::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(downloaded_bytes) => {
                callback(downloaded_bytes.len() as i64);
                buffer.extend_from_slice(&downloaded_bytes);
            }
            Err(err) => return Err(ClientError::RequestError(err)),
        }
    }
    Ok(buffer)
}
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to parse JSON response")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Failed to send request")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to send request")]
    UrlError(#[from] url::ParseError),

    #[error("HTTP error {status}: {body}")]
    Http { status: StatusCode, body: String },

    #[error("Decode error: {0}")]
    DecodeError(#[from] std::io::Error),
}
