use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const BASE_URL: &str = "https://api.telegram.org/bot";

#[derive(Debug)]
pub enum TelegramError {
    RequestError(ReqwestError),
    ApiError(String),
}

impl From<ReqwestError> for TelegramError {
    fn from(error: ReqwestError) -> Self {
        TelegramError::RequestError(error)
    }
}

#[derive(Debug, Deserialize)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub from: Option<User>,
    pub chat: Chat,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub chat_id: i64,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
}

pub struct Bot {
    token: String,
    client: Client,
}

impl Bot {
    pub fn new(token: String) -> Result<Self, TelegramError> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Bot { token, client })
    }

    fn api_url(&self, method: &str) -> String {
        format!("{}{}/{}", BASE_URL, self.token, method)
    }

    pub async fn get_updates(&self, offset: Option<i64>) -> Result<Vec<Update>, TelegramError> {
        let mut url = self.api_url("getUpdates");
        if let Some(offset) = offset {
            url = format!("{}?offset={}", url, offset);
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(ok) = response.get("ok").and_then(|v| v.as_bool()) {
            if !ok {
                return Err(TelegramError::ApiError(
                    response
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown API error")
                        .to_string(),
                ));
            }
        }

        let updates = response
            .get("result")
            .and_then(|v| serde_json::from_value::<Vec<Update>>(v.clone()).ok())
            .unwrap_or_default();

        Ok(updates)
    }

    pub async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<Message, TelegramError> {
        let url = self.api_url("sendMessage");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(ok) = response.get("ok").and_then(|v| v.as_bool()) {
            if !ok {
                return Err(TelegramError::ApiError(
                    response
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown API error")
                        .to_string(),
                ));
            }
        }

        let message = response
            .get("result")
            .and_then(|v| serde_json::from_value::<Message>(v.clone()).ok())
            .ok_or_else(|| {
                TelegramError::ApiError("Failed to parse message response".to_string())
            })?;

        Ok(message)
    }
}
