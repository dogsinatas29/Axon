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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelResponse {
    pub text: String,
    pub total_duration: Option<u64>,
    pub eval_count: Option<u64>,
    pub eval_duration: Option<u64>,
}

#[async_trait]
pub trait ModelDriver: Send + Sync {
    async fn generate(&self, prompt: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }
}

pub struct MockDriver;

#[async_trait]
impl ModelDriver for MockDriver {
    async fn generate(&self, _prompt: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ModelResponse {
            text: "Mock response from AXON Model Driver".to_string(),
            total_duration: None,
            eval_count: None,
            eval_duration: None,
        })
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

    async fn generate(&self, prompt: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", self.model_name, self.api_key);
        let body = serde_json::json!({"contents": [{"parts": [{"text": prompt}]}]});
        
        let mut retries = 5;
        loop {
            let response = self.client.post(&url).json(&body).send().await?;
            let status = response.status();
            let res_json: serde_json::Value = response.json().await?;
            
            if let Some(error) = res_json.get("error") {
                let err_msg = error["message"].as_str().unwrap_or("Unknown");
                if status.as_u16() == 429 || status.as_u16() == 503 || err_msg.contains("Quota exceeded") || err_msg.contains("Too Many Requests") || err_msg.contains("high demand") {
                    let mut wait_secs = 60.0;
                    if let Some(idx) = err_msg.find("Please retry in ") {
                        let substr = &err_msg[idx + 16..];
                        if let Some(end_idx) = substr.find("s.") {
                            if let Ok(parsed) = substr[..end_idx].parse::<f64>() {
                                wait_secs = parsed;
                            }
                        }
                    }
                    return Err(format!("QUOTA_WAIT:{}", wait_secs).into());
                }
                return Err(format!("Gemini API Error: {}", err_msg).into());
            }
            
            let text = res_json["candidates"][0]["content"]["parts"][0]["text"].as_str()
                .or_else(|| {
                    // v0.0.16: 일부 실험적 모델이 'parts' 없이 'thought'만 내보내는 경우 대응
                    tracing::warn!("Gemini Model produced empty parts. FinishReason: {:?}", res_json["candidates"][0]["finishReason"]);
                    None
                });

            match text {
                Some(t) => return Ok(ModelResponse {
                    text: t.to_string(),
                    total_duration: None,
                    eval_count: None,
                    eval_duration: None,
                }),
                None => {
                    if retries > 0 {
                        tracing::info!("🔄 Empty response from Gemini. Retrying... (Left: {})", retries);
                        retries -= 1;
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    return Err(format!("Gemini Output Exception: Model finished without generating text parts. FinishReason: {:?}", res_json["candidates"][0]["finishReason"]).into());
                }
            }
        }
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

    async fn generate(&self, prompt: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
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
        Ok(ModelResponse {
            text: text.to_string(),
            total_duration: None,
            eval_count: None,
            eval_duration: None,
        })
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

    async fn generate(&self, prompt: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model_name,
                "messages": [{"role": "user", "content": prompt}]
            })).send().await?;
        let res_json: serde_json::Value = response.json().await?;
        let text = res_json["choices"][0]["message"]["content"].as_str().ok_or("Failed OpenAI extraction")?;
        Ok(ModelResponse {
            text: text.to_string(),
            total_duration: None,
            eval_count: None,
            eval_duration: None,
        })
    }
}

pub struct OllamaDriver {
    base_url: String,
    model_name: String,
    client: reqwest::Client,
}

impl OllamaDriver {
    pub fn new(base_url: String, model_name: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model_name,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ModelDriver for OllamaDriver {
    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(url).send().await?;
        let res_json: serde_json::Value = response.json().await?;
        let mut models = Vec::new();
        if let Some(models_raw) = res_json["models"].as_array() {
            for m in models_raw {
                if let Some(name) = m["name"].as_str() {
                    models.push(name.to_string());
                }
            }
        }
        Ok(models)
    }

    async fn generate(&self, prompt: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url_str = format!("{}/api/generate", self.base_url);
        let url = reqwest::Url::parse(&url_str).map_err(|e| format!("Invalid Ollama URL '{}': {}", url_str, e))?;

        let model_lower = self.model_name.to_lowercase();
        let is_small = model_lower.contains("qwen") || model_lower.contains("gemma") || model_lower.contains("1.8b") || model_lower.contains("2b");
        
        let num_ctx = if is_small { 8192 } else { 32768 };
        let stop: Vec<String> = vec![]; // Remove problematic stop tokens

        let body = serde_json::json!({
            "model": self.model_name,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_ctx": num_ctx,
                "stop": stop,
                "temperature": 0.0
            }
        });

        tracing::debug!("📡 Ollama Request: URL={}, PromptSize={}", url, prompt.len());

        let response = self.client.post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama Connection/Builder Error: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            return Err(format!("Ollama API Error ({}): {}", status, err_body).into());
        }

        let res_json: serde_json::Value = response.json().await?;
        let text = res_json["response"].as_str().ok_or("Failed Ollama extraction")?;
        
        if text.trim().is_empty() {
            tracing::warn!("⚠️ Ollama returned an empty response field. Full JSON: {}", res_json);
        }

        Ok(ModelResponse {
            text: text.to_string(),
            total_duration: res_json["total_duration"].as_u64(),
            eval_count: res_json["eval_count"].as_u64(),
            eval_duration: res_json["eval_duration"].as_u64(),
        })
    }
}

