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
    client: reqwest::Client,
}

impl GeminiDriver {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ModelDriver for GeminiDriver {
    async fn generate(&self, prompt: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            self.api_key
        );

        let body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;

        let res_json: serde_json::Value = response.json().await?;
        
        // Basic extraction logic
        let text = res_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("Failed to extract text from Gemini response")?
            .to_string();

        Ok(text)
    }
}
