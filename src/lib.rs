#[derive(Debug, PartialEq)]
pub enum Error {
    CurlError(curl::Error),
    FromUtf8Error(std::string::FromUtf8Error),
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
