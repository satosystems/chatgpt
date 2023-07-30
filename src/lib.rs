#[derive(Debug, PartialEq)]
pub enum Error {
    CurlError(curl::Error),
    FromUtf8Error(std::string::FromUtf8Error),
    SerdeJsonError(String),
}

impl From<curl::Error> for Error {
    fn from(err: curl::Error) -> Self {
        Self::CurlError(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8Error(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJsonError(err.to_string())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Permission {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub allow_create_engine: bool,
    pub allow_sampling: bool,
    pub allow_logprobs: bool,
    pub allow_search_indices: bool,
    pub allow_view: bool,
    pub allow_fine_tuning: bool,
    pub organization: String,
    pub group: serde_json::Value,
    pub is_blocking: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
    pub permission: Vec<Permission>,
    pub root: String,
    pub parent: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct RequestBody {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

mod internal {
    use curl;

    pub fn init(api_key: &str, url: &str) -> Result<curl::easy::Easy, super::Error> {
        let mut easy = curl::easy::Easy::new();
        easy.url(url)?;
        let mut headers = curl::easy::List::new();
        headers.append(&format!("Authorization: Bearer {}", api_key))?;
        headers.append("Content-Type: application/json")?;
        headers.append("Accept: text/event-stream")?;
        easy.http_headers(headers)?;
        Ok(easy)
    }
}

pub mod ll {
    pub async fn list_models<F>(api_key: &str, f: F) -> Result<(), super::Error>
    where
        F: Fn(&[u8]),
    {
        let mut easy = super::internal::init(api_key, "https://api.openai.com/v1/models")?;
        easy.get(true)?;
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            f(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
        Ok(())
    }

    pub async fn completions<F>(
        api_key: &str,
        request_body: &super::RequestBody,
        f: F,
    ) -> Result<(), super::Error>
    where
        F: Fn(&[u8]),
    {
        let string_body = serde_json::to_string(request_body)?;
        let mut easy =
            super::internal::init(api_key, "https://api.openai.com/v1/chat/completions")?;
        easy.post(true)?;
        easy.post_fields_copy(string_body.as_bytes())?;
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            f(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
        Ok(())
    }
}

pub mod hl {
    pub async fn list_models(api_key: &str) -> Result<String, super::Error> {
        let amv = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let cloned_amv = amv.clone();
        super::ll::list_models(&api_key, move |data| {
            let mut v = cloned_amv.lock().unwrap();
            v.extend_from_slice(data)
        })
        .await?;
        let v = amv.lock().unwrap().clone();
        Ok(String::from_utf8(v)?)
    }
}

pub async fn list_models(api_key: &str) -> Result<Vec<ModelList>, Error> {
    let json = hl::list_models(api_key).await?;
    Ok(serde_json::from_str(&json)?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn init_handle() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not defined");
        let result = crate::internal::init(&api_key, "https://api.openai.com/v1/models");
        assert!(result.is_ok());
    }

    #[test]
    fn ll_list_models() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not defined");
        let future = crate::ll::list_models(&api_key, |_| {});
        let result = futures::executor::block_on(future);
        assert!(result.is_ok());
    }

    #[test]
    fn hl_list_models() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not defined");
        let future = crate::hl::list_models(&api_key);
        let result = futures::executor::block_on(future);
        assert!(result.is_ok());
    }
}
