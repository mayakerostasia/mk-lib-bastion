use anyhow::{anyhow, Error};
use async_nats::{HeaderMap, HeaderValue};

pub fn pop_headers(message: async_nats::Message) -> Result<HeaderMap, Error> {
    if message.headers.is_some() {
        let header_map = message.headers.unwrap();
        Ok(header_map)
    } else {
        Err(anyhow!("No Headers in this message"))
    }
}

fn _get_header_key(hm: HeaderMap, key: &str) -> Result<HeaderValue, Error> {
    match hm.get(key) {
        Some(hval) => Ok(hval.clone()),
        None => Err(anyhow!("No Header Key"))
    }
}

pub fn get_header_key(message: async_nats::Message, key: &str) -> Result<String, Error> {
    if let Ok(hm) = pop_headers(message) {
        if let Ok(hv) = _get_header_key(hm, key) {
            Ok(hv.as_str().to_string())
        } else { Ok("".to_string()) }
    } else { Ok("".to_string()) }
}
