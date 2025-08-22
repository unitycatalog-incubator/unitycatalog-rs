#![allow(unused, dead_code)]

#[cfg(feature = "recording")]
use std::collections::HashMap;
#[cfg(feature = "recording")]
use std::path::PathBuf;
#[cfg(feature = "recording")]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use aws::AmazonConfig;
use azure::AzureConfig;
use gcp::GoogleConfig;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Body, Client, IntoUrl, Method, RequestBuilder};
use serde::Serialize;

use self::azure::credential::AzureCredentialExt;
pub use self::token::{TemporaryToken, TokenCache};

pub mod aws;
pub mod azure;
mod backoff;
mod client;
mod config;
mod credential;
mod error;
pub mod gcp;

mod pagination;
mod retry;
mod token;
mod util;

pub use client::{Certificate, ClientConfigKey, ClientOptions};
pub use credential::*;
pub use error::*;
pub use pagination::stream_paginated;
pub use retry::RetryConfig;

#[derive(Clone)]
enum Credential {
    Aws(AmazonConfig),
    Google(GoogleConfig),
    Azure(AzureConfig),
    PersonalAccessToken(String),
    Unauthenticated,
}

#[derive(Clone)]
pub struct CloudClient {
    credential: Credential,
    http_client: Client,
    retry_config: RetryConfig,
    #[cfg(feature = "recording")]
    out_dir: Option<PathBuf>,
    #[cfg(feature = "recording")]
    recording_counter: std::sync::Arc<AtomicU64>,
}

impl CloudClient {
    pub fn new_aws<I, K, V>(options: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        let config = options
            .into_iter()
            .fold(
                aws::AmazonBuilder::new(),
                |builder, (key, value)| match key.as_ref().parse() {
                    Ok(k) => builder.with_config(k, value),
                    Err(_) => builder,
                },
            )
            .build()?;

        Ok(Self {
            http_client: config.client_options.client()?,
            retry_config: config.retry_config.clone(),
            credential: Credential::Aws(config),
            #[cfg(feature = "recording")]
            out_dir: None,
            #[cfg(feature = "recording")]
            recording_counter: std::sync::Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn new_google<I, K, V>(options: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        let config = options
            .into_iter()
            .fold(
                gcp::GoogleBuilder::new(),
                |builder, (key, value)| match key.as_ref().parse() {
                    Ok(k) => builder.with_config(k, value),
                    Err(_) => builder,
                },
            )
            .build()?;

        Ok(Self {
            http_client: config.client_options.client()?,
            retry_config: config.retry_config.clone(),
            credential: Credential::Google(config),
            #[cfg(feature = "recording")]
            out_dir: None,
            #[cfg(feature = "recording")]
            recording_counter: std::sync::Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn new_azure<I, K, V>(options: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        let config = options
            .into_iter()
            .fold(
                azure::AzureBuilder::new(),
                |builder, (key, value)| match key.as_ref().parse() {
                    Ok(k) => builder.with_config(k, value),
                    Err(_) => builder,
                },
            )
            .build()?;

        Ok(Self {
            http_client: config.client_options.client()?,
            retry_config: config.retry_config.clone(),
            credential: Credential::Azure(config),
            #[cfg(feature = "recording")]
            out_dir: None,
            #[cfg(feature = "recording")]
            recording_counter: std::sync::Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn new_with_token(token: impl ToString) -> Self {
        Self {
            http_client: Client::new(),
            retry_config: RetryConfig::default(),
            credential: Credential::PersonalAccessToken(token.to_string()),
            #[cfg(feature = "recording")]
            out_dir: None,
            #[cfg(feature = "recording")]
            recording_counter: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn new_unauthenticated() -> Self {
        Self {
            http_client: Client::new(),
            retry_config: RetryConfig::default(),
            credential: Credential::Unauthenticated,
            #[cfg(feature = "recording")]
            out_dir: None,
            #[cfg(feature = "recording")]
            recording_counter: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> CloudRequestBuilder {
        CloudRequestBuilder {
            builder: self.http_client.request(method, url),
            client: self.clone(),
            #[cfg(feature = "recording")]
            out_dir: self.out_dir.clone(),
        }
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::GET, url)
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::POST, url)
    }

    pub fn put<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::PUT, url)
    }

    pub fn delete<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::DELETE, url)
    }

    pub fn head<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::HEAD, url)
    }

    pub fn patch<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::PATCH, url)
    }

    pub fn options<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::OPTIONS, url)
    }

    pub fn trace<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::TRACE, url)
    }

    pub fn connect<U: IntoUrl>(&self, url: U) -> CloudRequestBuilder {
        self.request(Method::CONNECT, url)
    }

    #[cfg(feature = "recording")]
    pub fn set_recording_dir(&mut self, out_dir: PathBuf) -> Result<(), std::io::Error> {
        self.out_dir = Some(std::fs::canonicalize(out_dir)?);
        Ok(())
    }
}

pub struct CloudRequestBuilder {
    builder: RequestBuilder,
    client: CloudClient,
    #[cfg(feature = "recording")]
    out_dir: Option<PathBuf>,
}

impl CloudRequestBuilder {
    /// Add a `Header` to this Request.
    pub fn header<K, V>(mut self, key: K, value: V) -> CloudRequestBuilder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    /// Add a set of Headers to the existing ones on this Request.
    ///
    /// The headers will be merged in to any already set.
    pub fn headers(mut self, headers: HeaderMap) -> CloudRequestBuilder {
        self.builder = self.builder.headers(headers);
        self
    }

    /// Set the request body.
    pub fn body<T: Into<Body>>(mut self, body: T) -> CloudRequestBuilder {
        self.builder = self.builder.body(body);
        self
    }

    /// Enables a request timeout.
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished. It affects only this request and overrides
    /// the timeout configured using `ClientBuilder::timeout()`.
    pub fn timeout(mut self, timeout: Duration) -> CloudRequestBuilder {
        self.builder = self.builder.timeout(timeout);
        self
    }

    /// Modify the query string of the URL.
    ///
    /// Modifies the URL of this request, adding the parameters provided.
    /// This method appends and does not overwrite. This means that it can
    /// be called multiple times and that existing query parameters are not
    /// overwritten if the same key is used. The key will simply show up
    /// twice in the query string.
    /// Calling `.query(&[("foo", "a"), ("foo", "b")])` gives `"foo=a&foo=b"`.
    ///
    /// # Note
    /// This method does not support serializing a single key-value
    /// pair. Instead of using `.query(("key", "val"))`, use a sequence, such
    /// as `.query(&[("key", "val")])`. It's also possible to serialize structs
    /// and maps into a key-value pair.
    ///
    /// # Errors
    /// This method will fail if the object you provide cannot be serialized
    /// into a query string.
    pub fn query<T: Serialize + ?Sized>(mut self, query: &T) -> CloudRequestBuilder {
        self.builder = self.builder.query(query);
        self
    }

    /// Send a JSON body.
    ///
    /// # Errors
    ///
    /// Serialization can fail if `T`'s implementation of `Serialize` decides to
    /// fail, or if `T` contains a map with non-string keys.
    pub fn json<T: Serialize + ?Sized>(mut self, json: &T) -> CloudRequestBuilder {
        self.builder = self.builder.json(json);
        self
    }

    pub async fn send(mut self) -> Result<reqwest::Response> {
        match &self.client.credential {
            Credential::Azure(az) => {
                let credential = az.get_credential().await?;
                self.builder = self.builder.with_azure_authorization(&credential);
            }
            Credential::Aws(_aws) => {
                todo!()
            }
            Credential::Google(gcp) => {
                let credential = gcp.get_credential().await?;
                self.builder = self.builder.bearer_auth(&credential.bearer);
            }
            Credential::PersonalAccessToken(token) => {
                self.builder = self.builder.bearer_auth(token);
            }
            Credential::Unauthenticated => {
                // Do nothing
            }
        };
        #[cfg(not(feature = "recording"))]
        let response = self.builder.send().await?;
        #[cfg(feature = "recording")]
        let response = send_record(self).await?;
        Ok(response)
    }
}

#[cfg(feature = "recording")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestResponseInfo {
    pub request: RequestInfo,
    pub response: ResponseInfo,
}

#[cfg(feature = "recording")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestInfo {
    pub method: String,
    pub url_path: String,
    pub body: Option<String>,
}

#[cfg(feature = "recording")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseInfo {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[cfg(feature = "recording")]
async fn send_record(
    mut builder: CloudRequestBuilder,
) -> Result<reqwest::Response, reqwest::Error> {
    let Some(out_dir) = builder.out_dir else {
        return builder.builder.send().await;
    };
    let (client, request) = builder.builder.build_split();
    let request = request.expect("request to be valid");

    let request_info = RequestInfo {
        method: request.method().as_str().to_string(),
        url_path: {
            let url = request.url();
            match url.query() {
                Some(query) => format!("{}?{}", url.path(), query),
                None => url.path().to_string(),
            }
        },
        body: request
            .body()
            .and_then(|b| b.as_bytes().map(|b| String::from_utf8_lossy(b).to_string())),
    };

    let response = client.execute(request).await?;

    // Record the response
    let status = response.status().as_u16();
    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Get response body while preserving it for the caller
    let response_bytes = response.bytes().await?;
    let response_body = if response_bytes.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&response_bytes).to_string())
    };

    let response_info = ResponseInfo {
        status,
        headers: headers.clone(),
        body: response_body,
    };

    let recording = RequestResponseInfo {
        request: request_info,
        response: response_info,
    };

    let counter = builder
        .client
        .recording_counter
        .fetch_add(1, Ordering::SeqCst);
    let file_path = out_dir.join(format!("{:04}.json", counter));
    let file = std::fs::File::create(file_path).unwrap();
    serde_json::to_writer_pretty(file, &recording).unwrap();

    // Return a new response built from the recorded data
    let mut mock_response = http::Response::builder().status(status);
    for header in headers {
        mock_response = mock_response.header(header.0, header.1);
    }
    let mock_response = mock_response.body(response_bytes).unwrap();

    Ok(reqwest::Response::from(mock_response))
}

#[cfg(all(test, feature = "recording"))]
mod tests {
    use super::*;
    use mockito::ServerGuard;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_request_response_recording() {
        // Create a temporary directory for recordings
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Set up a mock server
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message": "Hello, World!"}"#)
            .create_async()
            .await;

        // Create a cloud client with recording enabled
        let mut client = CloudClient::new_unauthenticated();
        client.set_recording_dir(temp_path.clone()).unwrap();

        // Make a request
        let url = format!("{}/test", server.url());
        let response = client.get(&url).send().await.unwrap();

        // Verify the response is correct
        assert_eq!(response.status(), 200);
        let body = response.text().await.unwrap();
        assert_eq!(body, r#"{"message": "Hello, World!"}"#);

        // Verify that a recording file was created
        let recordings: Vec<_> = fs::read_dir(&temp_path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "json" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(recordings.len(), 1, "Expected exactly one recording file");

        // Read and verify the recording content
        let recording_content = fs::read_to_string(&recordings[0]).unwrap();
        let recording: RequestResponseInfo = serde_json::from_str(&recording_content).unwrap();

        // Verify request information
        assert_eq!(recording.request.method, "GET");
        assert_eq!(recording.request.url_path, "/test");
        assert_eq!(recording.request.body, None);

        // Verify response information
        assert_eq!(recording.response.status, 200);
        assert_eq!(
            recording.response.headers.get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(
            recording.response.body.as_ref().unwrap(),
            r#"{"message": "Hello, World!"}"#
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_recording_with_request_body() {
        // Create a temporary directory for recordings
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Set up a mock server
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/create")
            .with_status(201)
            .with_header("location", "/resource/123")
            .with_body(r#"{"id": 123, "status": "created"}"#)
            .create_async()
            .await;

        // Create a cloud client with recording enabled
        let mut client = CloudClient::new_unauthenticated();
        client.set_recording_dir(temp_path.clone()).unwrap();

        // Make a POST request with body
        let url = format!("{}/create", server.url());
        let response = client
            .post(&url)
            .json(&serde_json::json!({"name": "test resource"}))
            .send()
            .await
            .unwrap();

        // Verify the response
        assert_eq!(response.status(), 201);

        // Verify that a recording file was created
        let recordings: Vec<_> = fs::read_dir(&temp_path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "json" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(recordings.len(), 1);

        // Read and verify the recording content
        let recording_content = fs::read_to_string(&recordings[0]).unwrap();
        let recording: RequestResponseInfo = serde_json::from_str(&recording_content).unwrap();

        // Verify request information
        assert_eq!(recording.request.method, "POST");
        assert_eq!(recording.request.url_path, "/create");
        assert!(recording.request.body.is_some());
        assert!(recording.request.body.unwrap().contains("test resource"));

        // Verify response information
        assert_eq!(recording.response.status, 201);
        assert_eq!(
            recording.response.headers.get("location").unwrap(),
            "/resource/123"
        );
        assert!(recording.response.body.unwrap().contains("created"));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_counter_based_file_naming() {
        // Create a temporary directory for recordings
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Set up a mock server
        let mut server = mockito::Server::new_async().await;
        let mock1 = server
            .mock("GET", "/first")
            .with_status(200)
            .with_body("first response")
            .create_async()
            .await;
        let mock2 = server
            .mock("GET", "/second")
            .with_status(200)
            .with_body("second response")
            .create_async()
            .await;

        // Create a cloud client with recording enabled
        let mut client = CloudClient::new_unauthenticated();
        client.set_recording_dir(temp_path.clone()).unwrap();

        // Make multiple requests
        let url1 = format!("{}/first", server.url());
        let url2 = format!("{}/second", server.url());

        let _response1 = client.get(&url1).send().await.unwrap();
        let _response2 = client.get(&url2).send().await.unwrap();

        // Verify that files are named with incrementing counter
        let mut recordings: Vec<_> = fs::read_dir(&temp_path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "json" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        recordings.sort();
        assert_eq!(recordings.len(), 2);

        // Check that files are named 000000.json and 000001.json
        assert!(recordings[0].file_name().unwrap().to_str().unwrap() == "0000.json");
        assert!(recordings[1].file_name().unwrap().to_str().unwrap() == "0001.json");

        // Verify content matches the order of requests
        let first_content = fs::read_to_string(&recordings[0]).unwrap();
        let first_recording: RequestResponseInfo = serde_json::from_str(&first_content).unwrap();
        assert_eq!(first_recording.request.url_path, "/first");
        assert_eq!(
            first_recording.response.body.as_ref().unwrap(),
            "first response"
        );

        let second_content = fs::read_to_string(&recordings[1]).unwrap();
        let second_recording: RequestResponseInfo = serde_json::from_str(&second_content).unwrap();
        assert_eq!(second_recording.request.url_path, "/second");
        assert_eq!(
            second_recording.response.body.as_ref().unwrap(),
            "second response"
        );

        mock1.assert_async().await;
        mock2.assert_async().await;
    }

    #[tokio::test]
    async fn test_query_parameter_recording() {
        // Create a temporary directory for recordings
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Start a mock server
        let mut server = mockito::Server::new_async().await;

        // Create a mock that expects query parameters
        let mock = server
            .mock("GET", "/catalogs?max_results=10&page_token=abc123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"catalogs": []}"#)
            .create_async()
            .await;

        // Create a client with recording enabled
        let mut client = CloudClient::new_unauthenticated();
        client.set_recording_dir(temp_path.clone()).unwrap();

        // Make a request with query parameters
        let url = format!("{}/catalogs?max_results=10&page_token=abc123", server.url());
        let response = client.get(&url).send().await.unwrap();

        assert!(response.status().is_success());

        // Verify that the recording file was created
        let recordings: Vec<_> = fs::read_dir(&temp_path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "json" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(recordings.len(), 1);

        // Read and verify the recording content includes query parameters
        let recording_content = fs::read_to_string(&recordings[0]).unwrap();
        let recording: RequestResponseInfo = serde_json::from_str(&recording_content).unwrap();

        // Verify request information includes query parameters
        assert_eq!(recording.request.method, "GET");
        assert_eq!(
            recording.request.url_path,
            "/catalogs?max_results=10&page_token=abc123"
        );
        assert_eq!(recording.request.body, None);

        // Verify response information
        assert_eq!(recording.response.status, 200);
        assert_eq!(
            recording.response.body.as_ref().unwrap(),
            r#"{"catalogs": []}"#
        );

        mock.assert_async().await;
    }
}
