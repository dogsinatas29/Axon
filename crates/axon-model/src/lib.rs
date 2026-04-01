/*
 * AXON - The Automated Software Factory
 * Copyright (C) 2026 dogsinatas
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use async_trait::async_trait;

#[async_trait]
pub trait ModelDriver: Send + Sync {
    async fn generate(&self, prompt: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }
}

pub struct MockDriver;

#[async_trait]
impl ModelDriver for MockDriver {
    async fn generate(&self, _prompt: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("Mock response from AXON Model Driver".to_string())
    }
}

pub struct GeminiDriver {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
}

impl GeminiDriver {
    pub fn new(api_key: String, model_name: String) -> Self {
        Self { api_key, model_name, client: reqwest::Client::new() }
    }
}

pub struct ClaudeDriver {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
}

impl ClaudeDriver {
    pub fn new(api_key: String, model_name: String) -> Self {
        Self { api_key, model_name, client: reqwest::Client::new() }
    }
}

pub struct OpenAIDriver {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
}

impl OpenAIDriver {
    pub fn new(api_key: String, model_name: String) -> Self {
        Self { api_key, model_name, client: reqwest::Client::new() }
    }
}

#[async_trait]
impl ModelDriver for GeminiDriver {
    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", self.api_key);
        let response = self.client.get(url).send().await?;
        let res_json: serde_json::Value = response.json().await?;
        let mut models = Vec::new();
        if let Some(models_raw) = res_json["models"].as_array() {
            for m in models_raw {
                if let Some(name) = m["name"].as_str() {
                    models.push(name.strip_prefix("models/").unwrap_or(name).to_string());
                }
            }
        }
        Ok(models)
    }

    async fn generate(&self, prompt: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", self.model_name, self.api_key);
        let body = serde_json::json!({"contents": [{"parts": [{"text": prompt}]}]});
        let response = self.client.post(url).json(&body).send().await?;
        let res_json: serde_json::Value = response.json().await?;
        if let Some(error) = res_json.get("error") {
            return Err(format!("Gemini API Error: {}", error["message"].as_str().unwrap_or("Unknown")).into());
        }
        let text = res_json["candidates"][0]["content"]["parts"][0]["text"].as_str()
            .ok_or_else(|| { tracing::error!("Gemini Fallback. Raw: {}", res_json); "Failed extraction" })?;
        Ok(text.to_string())
    }
}

#[async_trait]
impl ModelDriver for ClaudeDriver {
    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Anthropic doesn't have a public ListModels API. Providing a curated list.
        Ok(vec![
            "claude-3-5-sonnet-20240620".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ])
    }

    async fn generate(&self, prompt: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": self.model_name,
                "max_tokens": 4096,
                "messages": [{"role": "user", "content": prompt}]
            })).send().await?;
        let res_json: serde_json::Value = response.json().await?;
        let text = res_json["content"][0]["text"].as_str().ok_or("Failed Claude extraction")?;
        Ok(text.to_string())
    }
}

#[async_trait]
impl ModelDriver for OpenAIDriver {
    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send().await?;
        let res_json: serde_json::Value = response.json().await?;
        let mut models = Vec::new();
        if let Some(data) = res_json["data"].as_array() {
            for m in data {
                if let Some(id) = m["id"].as_str() {
                    if id.starts_with("gpt") { models.push(id.to_string()); }
                }
            }
        }
        Ok(models)
    }

    async fn generate(&self, prompt: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model_name,
                "messages": [{"role": "user", "content": prompt}]
            })).send().await?;
        let res_json: serde_json::Value = response.json().await?;
        let text = res_json["choices"][0]["message"]["content"].as_str().ok_or("Failed OpenAI extraction")?;
        Ok(text.to_string())
    }
}
