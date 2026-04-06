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
  const PROVIDERS: &[(&str, &str)] = &[
    ("anthropic", "anthropic"),
    ("openai", "openai"),
    ("gemini", "gemini"),
    ("xai", "xai"),
  ];

  for (prefix, provider) in PROVIDERS {
    let slash_prefix = format!("/{prefix}/");
    let exact = format!("/{prefix}");
    if path.starts_with(slash_prefix.as_str()) || path == exact.as_str() {
      let clean = path.strip_prefix(exact.as_str()).unwrap_or(path);
      let clean = if clean.is_empty() { "/".to_string() } else { clean.to_string() };
      return (clean, Some(*provider));
    }
  }
  (path.to_string(), None)
}

/// Detect requested provider from model name in JSON request body
///
/// Inspects the `"model"` field of the JSON body:
/// - Models starting with `"claude"` → `"anthropic"`
/// - Models starting with `"gpt"`, `"chatgpt"`, or `OpenAI` o-series (`"oN..."` where N is a digit) → `"openai"`
/// - Models starting with `"gemini"` → `"gemini"`
/// - Models starting with `"grok"` → `"xai"`
/// - Otherwise → `None`
///
#[must_use]
pub fn detect_provider_from_model(body: &[u8]) -> Option<&'static str> {
  if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
    if let Some(model) = json.get("model").and_then(|m| m.as_str()) {
      if model.starts_with("claude") {
        return Some("anthropic");
      }
      // OpenAI: gpt-*, chatgpt-*, and o-series (o1, o2, o3, o4, o5, ...)
      let is_openai_o_series =
        model.starts_with('o') && model.chars().nth(1).is_some_and(|c| c.is_ascii_digit());
      if model.starts_with("gpt") || model.starts_with("chatgpt") || is_openai_o_series {
        return Some("openai");
      }
      if model.starts_with("gemini") {
        return Some("gemini");
      }
      if model.starts_with("grok") {
        return Some("xai");
      }
    }
  }
  None
}
