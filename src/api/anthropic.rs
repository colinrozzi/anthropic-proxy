use crate::bindings::theater::simple::http_client::{send_http, HttpRequest};
use crate::bindings::theater::simple::runtime::log;
use crate::bindings::theater::simple::timing;
use crate::types::api::{
    AnthropicCompletionRequest, AnthropicCompletionResponse, AnthropicError, AnthropicModelInfo,
};
use crate::types::state::RetryConfig;

use serde_json::Value;

/// Client for interacting with the Anthropic API
pub struct AnthropicClient {
    /// Anthropic API key
    api_key: String,

    /// Base URL for the API
    base_url: String,

    /// API version to use
    api_version: String,
}

impl AnthropicClient {
    /// Create a new Anthropic client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            api_version: "2023-06-01".to_string(),
        }
    }

    /// Check if a status code indicates a retryable error
    fn is_retryable_error(status: u16) -> bool {
        match status {
            429 => true, // Rate limit exceeded
            502 => true, // Bad gateway
            503 => true, // Service unavailable
            504 => true, // Gateway timeout
            529 => true, // Service overloaded (Anthropic specific)
            _ => false,
        }
    }

    /// Execute an HTTP request with exponential backoff retry logic
    fn execute_with_retry(
        &self,
        request: &HttpRequest,
        retry_config: &RetryConfig,
    ) -> Result<crate::bindings::theater::simple::http_client::HttpResponse, AnthropicError> {
        let start_time = timing::now();
        let mut current_delay = retry_config.initial_delay_ms;
        let mut attempt = 0;

        loop {
            attempt += 1;
            
            log(&format!("HTTP request attempt {}/{}", attempt, retry_config.max_retries + 1));

            // Send the request
            let response = match send_http(request) {
                Ok(resp) => resp,
                Err(e) => {
                    log(&format!("HTTP request failed: {}", e));
                    if attempt > retry_config.max_retries {
                        return Err(AnthropicError::HttpError(e));
                    }
                    
                    // Check if we've exceeded the total timeout
                    let elapsed = timing::now() - start_time;
                    if elapsed >= retry_config.max_total_timeout_ms as u64 {
                        log("Total retry timeout exceeded");
                        return Err(AnthropicError::HttpError(e));
                    }
                    
                    // Wait before retrying
                    log(&format!("Retrying after {} ms due to HTTP error", current_delay));
                    let _ = timing::sleep(current_delay as u64);
                    current_delay = std::cmp::min(
                        (current_delay as f64 * retry_config.backoff_multiplier) as u32,
                        retry_config.max_delay_ms
                    );
                    continue;
                }
            };

            // Check if we got a successful response
            if response.status == 200 {
                log(&format!("Request successful on attempt {}", attempt));
                return Ok(response);
            }

            // Check if this is a retryable error
            if !Self::is_retryable_error(response.status) {
                log(&format!("Non-retryable error: {}", response.status));
                return Ok(response); // Return the error response to be handled by caller
            }

            // Check if we've exhausted our retries
            if attempt > retry_config.max_retries {
                log(&format!("Max retries ({}) exceeded", retry_config.max_retries));
                return Ok(response);
            }

            // Check if we've exceeded the total timeout
            let elapsed = timing::now() - start_time;
            if elapsed >= retry_config.max_total_timeout_ms as u64 {
                log("Total retry timeout exceeded");
                return Ok(response);
            }

            // Log the retry attempt
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            log(&format!(
                "Retryable error {} on attempt {}: {}",
                response.status, attempt, message
            ));
            log(&format!("Retrying after {} ms", current_delay));

            // Wait before retrying
            let _ = timing::sleep(current_delay as u64);
            
            // Update delay for next attempt (exponential backoff)
            current_delay = std::cmp::min(
                (current_delay as f64 * retry_config.backoff_multiplier) as u32,
                retry_config.max_delay_ms
            );
        }
    }

    /// List available models from the Anthropic API
    pub fn list_models(&self) -> Result<Vec<AnthropicModelInfo>, AnthropicError> {
        log("Listing available Anthropic models");

        let request = HttpRequest {
            method: "GET".to_string(),
            uri: format!("{}/models", self.base_url),
            headers: vec![
                ("x-api-key".to_string(), self.api_key.clone()),
                ("anthropic-version".to_string(), self.api_version.clone()),
                ("content-type".to_string(), "application/json".to_string()),
            ],
            body: None,
        };

        // Use default retry config for model listing (lighter retries)
        let retry_config = RetryConfig {
            max_retries: 2,
            initial_delay_ms: 500,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            max_total_timeout_ms: 15000,
        };

        let response = self.execute_with_retry(&request, &retry_config)?;

        // Check status code
        if response.status != 200 {
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            return Err(AnthropicError::ApiError {
                status: response.status,
                message,
            });
        }

        // Parse the response
        let body = response
            .body
            .ok_or_else(|| AnthropicError::InvalidResponse("No response body".to_string()))?;

        log(&format!(
            "Models API response: {}",
            String::from_utf8_lossy(&body)
        ));

        let response_data: Value = serde_json::from_slice(&body)?;

        // Extract the models
        let mut models = Vec::new();

        if let Some(data) = response_data.get("data").and_then(|d| d.as_array()) {
            for model_data in data {
                if let (Some(id), Some(name)) = (
                    model_data.get("id").and_then(|v| v.as_str()),
                    model_data.get("display_name").and_then(|v| v.as_str()),
                ) {
                    let max_tokens = AnthropicModelInfo::get_max_tokens(id);
                    let pricing = AnthropicModelInfo::get_pricing(id);

                    models.push(AnthropicModelInfo {
                        id: id.to_string(),
                        display_name: name.to_string(),
                        max_tokens,
                        provider: "anthropic".to_string(),
                        pricing: Some(pricing),
                    });
                }
            }
        }

        Ok(models)
    }

    /// Generate a completion using the Anthropic API with retry logic
    pub fn generate_completion(
        &self,
        request: AnthropicCompletionRequest,
        retry_config: &RetryConfig,
    ) -> Result<AnthropicCompletionResponse, AnthropicError> {
        log("Generating completion with Anthropic API");

        // Create the HTTP request
        let http_request = HttpRequest {
            method: "POST".to_string(),
            uri: format!("{}/messages", self.base_url),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("x-api-key".to_string(), self.api_key.clone()),
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ],
            body: Some(serde_json::to_vec(&request)?),
        };

        // Execute with retry logic
        let response = self.execute_with_retry(&http_request, retry_config)?;

        // Check status code
        if response.status != 200 {
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            return Err(AnthropicError::ApiError {
                status: response.status,
                message,
            });
        }

        // Parse the response
        let body = response
            .body
            .ok_or_else(|| AnthropicError::InvalidResponse("No response body".to_string()))?;

        log(&format!("Got response: {}", String::from_utf8_lossy(&body)));

        serde_json::from_slice(&body).map_err(|e| AnthropicError::InvalidResponse(e.to_string()))
    }
}
