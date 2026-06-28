//! Web search execution for the built-in `web_search` tool.
//!
//! When the upstream model returns a `web_search` tool_use, cc-switch intercepts it,
//! calls a search API (Exa or Tavily), and injects the results as a `tool_result`
//! back into the conversation. This enables 3P users to have web search capability.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub text: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WebSearchError {
    #[error("Web search not configured: set web_search_provider and web_search_api_key")]
    NotConfigured,
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
    #[error("API request failed: {0}")]
    RequestFailed(String),
    #[error("API response parse error: {0}")]
    ParseError(String),
}

/// Execute a web search query using the configured provider.
pub async fn execute_web_search(
    query: &str,
    provider: &str,
    api_key: &str,
) -> Result<Vec<WebSearchResult>, WebSearchError> {
    match provider.to_lowercase().as_str() {
        "exa" => execute_exa_search(query, api_key).await,
        "tavily" => execute_tavily_search(query, api_key).await,
        _ => Err(WebSearchError::UnsupportedProvider(provider.to_string())),
    }
}

/// Execute search via Exa API (https://api.exa.ai/search)
async fn execute_exa_search(query: &str, api_key: &str) -> Result<Vec<WebSearchResult>, WebSearchError> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.exa.ai/search")
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "numResults": 5,
            "contents": {
                "text": {
                    "maxCharacters": 1000
                }
            }
        }))
        .send()
        .await
        .map_err(|e| WebSearchError::RequestFailed(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(WebSearchError::RequestFailed(format!(
            "Exa API returned {}: {}",
            status, error_text
        )));
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| WebSearchError::ParseError(e.to_string()))?;

    let results = json["results"]
        .as_array()
        .ok_or_else(|| WebSearchError::ParseError("Missing results array".to_string()))?
        .iter()
        .filter_map(|r| {
            Some(WebSearchResult {
                title: r["title"].as_str().unwrap_or("Untitled").to_string(),
                url: r["url"].as_str().unwrap_or("").to_string(),
                text: r["text"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect();

    Ok(results)
}

/// Execute search via Tavily API (https://api.tavily.com/search)
async fn execute_tavily_search(
    query: &str,
    api_key: &str,
) -> Result<Vec<WebSearchResult>, WebSearchError> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.tavily.com/search")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "api_key": api_key,
            "query": query,
            "max_results": 5,
            "include_raw_content": false
        }))
        .send()
        .await
        .map_err(|e| WebSearchError::RequestFailed(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(WebSearchError::RequestFailed(format!(
            "Tavily API returned {}: {}",
            status, error_text
        )));
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| WebSearchError::ParseError(e.to_string()))?;

    let results = json["results"]
        .as_array()
        .ok_or_else(|| WebSearchError::ParseError("Missing results array".to_string()))?
        .iter()
        .filter_map(|r| {
            Some(WebSearchResult {
                title: r["title"].as_str().unwrap_or("Untitled").to_string(),
                url: r["url"].as_str().unwrap_or("").to_string(),
                text: r["content"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect();

    Ok(results)
}

/// Format search results as a tool_result content string for the model.
pub fn format_search_results(results: &[WebSearchResult]) -> String {
    if results.is_empty() {
        return "No search results found.".to_string();
    }

    let mut output = String::new();
    for (i, result) in results.iter().enumerate() {
        output.push_str(&format!(
            "[{}] {}\nURL: {}\n{}\n\n",
            i + 1,
            result.title,
            result.url,
            result.text
        ));
    }
    output
}
