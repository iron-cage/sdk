//! Provider detection from request path and model name

/// Strip provider prefix from path if present
///
/// Recognizes `/anthropic/...` and `/openai/...` prefixes.
/// Returns the cleaned path and the detected provider name.
///
/// # Examples
///
/// ```
/// use iron_llm_core::provider::strip_provider_prefix;
///
/// let (path, provider) = strip_provider_prefix("/anthropic/v1/messages");
/// assert_eq!(path, "/v1/messages");
/// assert_eq!(provider, Some("anthropic"));
///
/// let (path, provider) = strip_provider_prefix("/v1/chat/completions");
/// assert_eq!(path, "/v1/chat/completions");
/// assert_eq!(provider, None);
/// ```
#[must_use]
pub fn strip_provider_prefix(path: &str) -> (String, Option<&'static str>) {
  if path.starts_with("/anthropic/") || path.starts_with("/anthropic") {
    let clean = path.strip_prefix("/anthropic").unwrap_or(path);
    let clean = if clean.is_empty() {
      "/".to_string()
    } else {
      clean.to_string()
    };
    (clean, Some("anthropic"))
  } else if path.starts_with("/openai/") || path.starts_with("/openai") {
    let clean = path.strip_prefix("/openai").unwrap_or(path);
    let clean = if clean.is_empty() {
      "/".to_string()
    } else {
      clean.to_string()
    };
    (clean, Some("openai"))
  } else {
    (path.to_string(), None)
  }
}

/// Detect requested provider from model name in JSON request body
///
/// Inspects the `"model"` field of the JSON body:
/// - Models starting with `"claude"` → `"anthropic"`
/// - Models starting with `"gpt"`, `"o1"`, or `"o3"` → `"openai"`
/// - Otherwise → `None`
#[must_use]
pub fn detect_provider_from_model(body: &[u8]) -> Option<&'static str> {
  if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
    if let Some(model) = json.get("model").and_then(|m| m.as_str()) {
      if model.starts_with("claude") {
        return Some("anthropic");
      }
      if model.starts_with("gpt") || model.starts_with("o1") || model.starts_with("o3") {
        return Some("openai");
      }
    }
  }
  None
}
