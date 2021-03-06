use hex::encode as hex_encode;
use hmac::{Hmac, Mac, NewMac};
use crate::errors::*;
use reqwest::StatusCode;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT, CONTENT_TYPE};
use sha2::Sha256;
use std::io::Read;

#[derive(Clone)]
pub struct Client {
    api_key: String,
    secret_key: String,
    host: String,
}

impl Client {
    pub fn new(api_key: Option<String>, secret_key: Option<String>, host: String) -> Self {
        Client {
            api_key: api_key.unwrap_or_else(|| "".into()),
            secret_key: secret_key.unwrap_or_else(|| "".into()),
            host,
        }
    }

    pub fn get_signed(&self, endpoint: &str, request: &str) -> Result<String> {
        let url = self.sign_request(endpoint, request);
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;

        self.handler(response)
    }

    pub fn post_signed(&self, endpoint: &str, request: &str) -> Result<String> {
        let url = self.sign_request(endpoint, request);
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;

        self.handler(response)
    }

    pub fn delete_signed(&self, endpoint: &str, request: &str) -> Result<String> {
        let url = self.sign_request(endpoint, request);
        let client = reqwest::blocking::Client::new();
        let response = client
            .delete(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;

        self.handler(response)
    }

    pub fn get(&self, endpoint: &str, request: &str) -> Result<String> {
        let mut url: String = format!("{}{}", self.host, endpoint);
        if !request.is_empty() {
            url.push_str(format!("?{}", request).as_str());
        }
        println!("get: {}", url);

        let response = reqwest::blocking::get(url.as_str())?;
        self.handler(response)
    }

    pub fn post(&self, endpoint: &str) -> Result<String> {
        let url: String = format!("{}{}", self.host, endpoint);
        println!("post: {}", url);

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(url.as_str())
            .headers(self.build_headers(false)?)
            .send()?;

        self.handler(response)
    }

    pub fn put(&self, endpoint: &str, listen_key: &str) -> Result<String> {
        let url: String = format!("{}{}", self.host, endpoint);
        let data: String = format!("listenKey={}", listen_key);

        let client = reqwest::blocking::Client::new();
        println!("put: {}", url);
        let response = client
            .put(url.as_str())
            .headers(self.build_headers(false)?)
            .body(data)
            .send()?;

        self.handler(response)
    }

    pub fn delete(&self, endpoint: &str, listen_key: &str) -> Result<String> {
        let url: String = format!("{}{}", self.host, endpoint);
        let data: String = format!("listenKey={}", listen_key);

        let client = reqwest::blocking::Client::new();
        println!("delete: {}", url);
        let response = client
            .delete(url.as_str())
            .headers(self.build_headers(false)?)
            .body(data)
            .send()?;
            println!("{}", url);

        self.handler(response)
    }

    // Request must be signed
    fn sign_request(&self, endpoint: &str, request: &str) -> String {
        let mut signed_key = Hmac::<Sha256>::new_varkey(self.secret_key.as_bytes()).unwrap();
        signed_key.update(request.as_bytes());
        let signature = hex_encode(signed_key.finalize().into_bytes());
        let request_body: String = format!("{}&signature={}", request, signature);
        let url: String = format!("{}{}?{}", self.host, endpoint, request_body);
        println!("{}", url);

        url
    }

    fn build_headers(&self, content_type: bool) -> Result<HeaderMap> {
        let mut custom_headers = HeaderMap::new();

        custom_headers.insert(USER_AGENT, HeaderValue::from_static("binance-rs"));
        if content_type {
            custom_headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }
        custom_headers.insert(
            HeaderName::from_static("x-mbx-apikey"),
            HeaderValue::from_str(self.api_key.as_str())?,
        );

        Ok(custom_headers)
    }

    fn handler(&self, mut response: Response) -> Result<String> {
        match response.status() {
            StatusCode::OK => {
                let mut body = String::new();
                response.read_to_string(&mut body)?;
                Ok(body)
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                bail!("Internal Server Error");
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                bail!("Service Unavailable");
            }
            StatusCode::UNAUTHORIZED => {
                bail!("Unauthorized");
            }
            StatusCode::BAD_REQUEST => {
                let error: BinanceContentError = response.json()?;

                Err(ErrorKind::BinanceError(error).into())
            }
            s => {
                bail!(format!("Received response: {:?}", s));
            }
        }
    }
}
