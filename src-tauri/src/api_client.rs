use reqwest_middleware::{ClientWithMiddleware, Error};
use tauri_plugin_http::reqwest::Response;

pub(crate) struct ApiClient {
    pub base_url: String,
    pub client: ClientWithMiddleware,
}

impl ApiClient {
    pub fn new(base_url: String, client: ClientWithMiddleware) -> Self {
        Self { base_url, client }
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub async fn post(&self, path: &str, body: Vec<u8>) -> Result<Response, Error> {
        self.client.post(self.build_url(path))
            .body(body)
            .send()
            .await
    }

    pub async fn post_with_headers(&self, path: &str, body: Vec<u8>, session_id: &str) -> Result<Response, Error> {
        self.client.post(self.build_url(path))
            .body(body)
            .header("session-id", session_id)
            .send()
            .await
    }
    
    pub async fn post_empty(&self, path: &str) -> Result<Response, Error> {
        self.client.post(self.build_url(path))
            .send()
            .await
    }

    pub async fn get(&self, path: &str) -> Result<Response, Error> {
        self.client.get(self.build_url(path))
            .send()
            .await
    }
    
    pub async fn get_with_body(&self, path: &str, body: Vec<u8>) -> Result<Response, Error> {
        self.client.get(self.build_url(path))
            .body(body)
            .send()
            .await
    }

    pub async fn put(&self, path: &str, body: Vec<u8>) -> Result<Response, Error> {
        self.client.put(self.build_url(path))
            .body(body)
            .send()
            .await
    }

    pub async fn delete(&self, path: &str) -> Result<Response, Error> {
        self.client.delete(self.build_url(path))
            .send()
            .await
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            client: ClientWithMiddleware::new(crate::http_client::https_client(), vec![]),
        }
    }
}
