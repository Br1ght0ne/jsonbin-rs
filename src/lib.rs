use isahc::http::{self, request::Builder as RequestBuilder};
use isahc::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

const API_ROOT: &str = "https://jsonbin.org";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to create request: {0:?}")]
    CreateRequest(#[from] http::Error),
    #[error("failed to send request: {0:?}")]
    SendRequest(#[from] isahc::Error),
    #[error("failed to parse JSON response: {0:?}")]
    JSONResponse(#[from] serde_json::Error),
    // TODO: handle 401 Unauthorized
    // #[error("invalid authorization token")]
    // AuthorizationToken,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Bin {
    id: String,
    data: Value,
}

pub struct JSONBin {
    token: String,
}

impl JSONBin {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    fn authorize<'a>(&'a self, request: &'a mut RequestBuilder) -> &'a mut RequestBuilder {
        request.header("authorization", format!("token {}", self.token))
    }

    pub fn create<T: Serialize>(&self, path: &str, data: &T) -> Result<bool> {
        let mut endpoint = Request::post(format!("{}/me/{}", API_ROOT, path));
        let request = self
            .authorize(&mut endpoint)
            .body(serde_json::to_string(data)?)
            .map_err(Error::from)?;
        let response = request.send().map_err(Error::from)?;
        Ok(response.status().is_success())
    }

    pub fn read<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let mut endpoint = Request::get(format!("{}/me/{}", API_ROOT, path));
        let request = self
            .authorize(&mut endpoint)
            .body(())
            .map_err(Error::from)?;
        let mut response = request.send().map_err(Error::from)?;
        response.json().map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Stats {
        count: u32,
    }

    #[test]
    fn it_works() -> Result<()> {
        let api = JSONBin::new(env!("JSONBIN_ORG_TOKEN"));
        let path = "rust_api_test";
        let stats = Stats { count: 5 };
        assert!(api.create(path, &stats)?);
        assert_eq!(stats, api.read(path)?);
        Ok(())
    }
}
