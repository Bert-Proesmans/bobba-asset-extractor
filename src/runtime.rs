use std::path::PathBuf;

use reqwest::{blocking::Client, header};

#[derive(Debug)]
pub enum Error {
    HttpClient(reqwest::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::HttpClient(ref reqwest) => write!(f, "Http client error: {}", reqwest),
        }
    }
}

impl std::error::Error for Error {}

pub(crate) struct RuntimeData {
    base_data_path: PathBuf,
    revision: String,
}

impl RuntimeData {
    pub fn new(base_data_path: PathBuf, revision: String) -> Self {
        RuntimeData {
            base_data_path,
            revision,
        }
    }

    pub fn create_web_client() -> Result<Client, Error> {
        let mut request_headers = header::HeaderMap::new();
        // WARN; User agent is required because Habbo blocks other asset requests!
        // This header emulates a request from Firefox, but might need tweaking to work in the future.
        request_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:74.0) Gecko/20100101 Firefox/74.0",
            ),
        );
        request_headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );

        Client::builder()
            .default_headers(request_headers)
            .gzip(true)
            .build()
            .map_err(Error::HttpClient)
    }

    pub fn get_data_path(&self) -> PathBuf {
        self.base_data_path.join(&self.revision)
    }
}
