// 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

use std::time::Duration;

use hyper::{Method, StatusCode};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    error::Result,
    genai::{ChatRequest, ChatResponse},
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct AiProvider {
    pub base_url: String,
    pub api_key: String,
}

impl AiProvider {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    /// Создает экземпляр AiProvider из переменных окружения.
    ///
    /// Переменные окружения:
    /// - `AI_PROVIDER_BASE_URL` — базовый URL API провайдера.
    /// - `AI_PROVIDER_API_KEY` — API ключ для доступа к API.
    ///
    /// # Паника
    /// Метод вызовет `panic!`, если одна из переменных окружения не установлена.
    pub fn from_env() -> Self {
        let _ = dotenv::dotenv().ok(); // Загружаем .env, если он есть

        let base_url = std::env::var("AI_PROVIDER_BASE_URL")
            .expect("Переменная окружения AI_PROVIDER_BASE_URL не установлена");
        let api_key = std::env::var("AI_PROVIDER_API_KEY")
            .expect("Переменная окружения AI_PROVIDER_API_KEY не установлена");

        Self::new(&base_url, &api_key)
    }

    async fn call_api<T: DeserializeOwned, R: Serialize>(
        &self,
        method: Method,
        path: &str,
        payload: &R,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);

        let mut attempts = 0;
        loop {
            let response = reqwest::Client::new()
                .request(method.clone(), &url)
                .timeout(DEFAULT_TIMEOUT)
                .bearer_auth(&self.api_key)
                .json(payload)
                .send()
                .await?;

            if response.status() == 429 && attempts < 9 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
                continue;
            }

            return Ok(response.error_for_status()?.json::<T>().await?);
        }

        unreachable!()
    }

    pub async fn chat(&self, r: &ChatRequest) -> Result<ChatResponse> {
        self.call_api(Method::POST, "/chat/completions", r).await
    }
}
