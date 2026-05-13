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
    pub thought: Option<String>,
    pub total_duration: Option<u64>,
    pub eval_count: Option<u64>,
    pub eval_duration: Option<u64>,
}

#[async_trait]
pub trait ModelDriver: Send + Sync {
    fn id(&self) -> String;
    async fn generate(&self, prompt: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>>;
    
    // v0.0.28: Enhanced prompt management for strict contracts
    async fn generate_with_system(&self, system: String, user: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        let combined = format!("{}\n\n{}", system, user);
        self.generate(combined).await
    }

    async fn generate_with_context(&self, prompt: String, _context_size: usize) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.generate(prompt).await
    }

    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }
}

pub struct MockDriver;

#[async_trait]
impl ModelDriver for MockDriver {
    fn id(&self) -> String { "mock".to_string() }
    async fn generate(&self, _prompt: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ModelResponse {
            text: "Mock response from AXON Model Driver".to_string(),
            thought: None,
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
        Self { api_key, model_name, client: reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build().unwrap_or_default() }
    }
}

pub struct ClaudeDriver {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
}

impl ClaudeDriver {
    pub fn new(api_key: String, model_name: String) -> Self {
        Self { api_key, model_name, client: reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build().unwrap_or_default() }
    }
}

pub struct OpenAIDriver {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
}

impl OpenAIDriver {
    pub fn new(api_key: String, model_name: String) -> Self {
        Self { api_key, model_name, client: reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build().unwrap_or_default() }
    }
}

#[async_trait]
impl ModelDriver for GeminiDriver {
    fn id(&self) -> String { self.model_name.clone() }
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
        let body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {
                "temperature": 0.0,
                "topP": 1.0,
                "maxOutputTokens": 4096,
                "stopSequences": ["<JSON_END>", "###"]
            }
        });
        
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
                    // v0.0.28: 일부 실험적 모델이 'parts' 없이 'thought'만 내보내는 경우 대응
                    tracing::warn!("Gemini Model produced empty parts. FinishReason: {:?}", res_json["candidates"][0]["finishReason"]);
                    None
                });

            match text {
                Some(t) => return Ok(ModelResponse {
                    text: t.to_string(),
                    thought: None,
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
    fn id(&self) -> String { self.model_name.clone() }
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
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.0,
                "stop_sequences": ["<JSON_END>", "###"]
            })).send().await?;
        let res_json: serde_json::Value = response.json().await?;
        let text = res_json["content"][0]["text"].as_str().ok_or("Failed Claude extraction")?;
        Ok(ModelResponse {
            text: text.to_string(),
            thought: None,
            total_duration: None,
            eval_count: None,
            eval_duration: None,
        })
    }
}

#[async_trait]
impl ModelDriver for OpenAIDriver {
    fn id(&self) -> String { self.model_name.clone() }
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
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.0,
                "stop": ["<JSON_END>", "###"]
            })).send().await?;
        let res_json: serde_json::Value = response.json().await?;
        let text = res_json["choices"][0]["message"]["content"].as_str().ok_or("Failed OpenAI extraction")?;
        Ok(ModelResponse {
            text: text.to_string(),
            thought: None,
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

static OLLAMA_MUTEX: std::sync::LazyLock<tokio::sync::Mutex<()>> = std::sync::LazyLock::new(|| {
    tokio::sync::Mutex::new(())
});

impl OllamaDriver {
    pub fn new(base_url: String, model_name: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model_name,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(180)) // Increased for local LLM swap hell
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl ModelDriver for OllamaDriver {
    fn id(&self) -> String { self.model_name.clone() }
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
        // v0.0.28: Default to empty system prompt and dynamic context
        self.generate_with_context(prompt, 0).await
    }

    async fn generate_with_context(&self, prompt: String, context_size: usize) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.generate_internal("".to_string(), prompt, context_size).await
    }

    async fn generate_with_system(&self, system: String, user: String) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.generate_internal(system, user, 0).await
    }
}

impl OllamaDriver {
    async fn generate_internal(&self, system: String, user: String, forced_context: usize) -> Result<ModelResponse, Box<dyn std::error::Error + Send + Sync>> {
        // [VRAM PROTECTION] Global Inference Lock (Admission Control)
        let _lock = OLLAMA_MUTEX.lock().await;

        let url_str = format!("{}/api/generate", self.base_url);
        let url = reqwest::Url::parse(&url_str).map_err(|e| format!("Invalid Ollama URL '{}': {}", url_str, e))?;

        let is_skeleton = user.contains("Node Definitions") || user.contains("IR Spec") || user.contains("Skeleton");
        let mut current_ctx = if forced_context > 0 {
            forced_context
        } else if is_skeleton {
            16384
        } else {
            8192
        };
        let mut current_user_prompt = user;
        let mut attempts = 0;

        loop {
            attempts += 1;
            
            // v0.0.28: Unified KISS Prompt - No separate System role to avoid Ollama parser bugs
            let full_prompt = if system.is_empty() {
                current_user_prompt.clone()
            } else {
                format!("### INSTRUCTIONS ###\n{}\n\n### INPUT ###\n{}", system, current_user_prompt)
            };

            let body = serde_json::json!({
                "model": self.model_name,
                "prompt": full_prompt,
                "stream": false,
                "options": {
                    "num_ctx": current_ctx,
                    "num_predict": 4096,
                    "stop": [],
                    "keep_alive": "30m",
                    "temperature": 0.1,
                    "top_p": 0.8
                }
            });

            tracing::info!("📡 [VRAM_SAFE] Ollama Request: Model={}, Context={}, Attempt={}/3", 
                self.model_name, current_ctx, attempts);

            let response_future = self.client.post(url.clone())
                .json(&body)
                .send();
            
            let response = tokio::time::timeout(std::time::Duration::from_secs(120), response_future).await;

            match response {
                Ok(Ok(res)) => {
                    let status = res.status();
                    let raw_text = res.text().await.unwrap_or_default();
                    
                    if !status.is_success() {
                        tracing::warn!("⚠️ Ollama API Error (Status {}): {}. Sleeping 2s...", status, raw_text);
                    } else if let Ok(res_json) = serde_json::from_str::<serde_json::Value>(&raw_text) {
                        let response_text = res_json["response"].as_str().unwrap_or_default().to_string();
                        
                        if response_text.trim().is_empty() {
                            tracing::warn!("⚠️ [EMPTY_RESPONSE] Response is empty. Sleeping 2s for retry...");
                        } else {
                            return Ok(ModelResponse {
                                text: response_text,
                                thought: None,
                                total_duration: res_json["total_duration"].as_u64(),
                                eval_count: res_json["eval_count"].as_u64(),
                                eval_duration: res_json["eval_duration"].as_u64(),
                            });
                        }
                    }
                }
                _ => {
                    tracing::warn!("⚠️ Ollama Timeout or Connection Error. Sleeping 2s...");
                }
            }

            if attempts >= 3 { break; }
            
            // v0.0.28: VRAM Cool-down & Gentle Context Degradation
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            current_ctx = (current_ctx * 3 / 4).max(2048);
            let new_len = (current_user_prompt.chars().count() * 7) / 8;
            current_user_prompt = current_user_prompt.chars().take(new_len).collect();
        }

        Err(format!("Ollama generation failed after 3 attempts with progressive degradation").into())
    }
}

