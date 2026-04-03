use anyhow::{bail, Context, Result};

use crate::config::TtsConfig;
use crate::util::truncate_with_ellipsis;

#[cfg(test)]
fn test_tts_override() -> &'static std::sync::Mutex<Option<std::result::Result<Vec<u8>, String>>> {
    static OVERRIDE: std::sync::OnceLock<
        std::sync::Mutex<Option<std::result::Result<Vec<u8>, String>>>,
    > = std::sync::OnceLock::new();
    OVERRIDE.get_or_init(|| std::sync::Mutex::new(None))
}

#[cfg(test)]
pub(crate) fn lock_test_synthesis() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
pub(crate) fn set_test_synthesis_result(result: std::result::Result<Vec<u8>, String>) {
    *test_tts_override()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(result);
}

fn resolve_tts_api_key(config: &TtsConfig, api_key_override: Option<&str>) -> Result<String> {
    if let Some(value) = config
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        return Ok(value.to_string());
    }

    if let Ok(value) = std::env::var("TTS_API_KEY") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    if let Some(value) = api_key_override
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        return Ok(value.to_string());
    }

    for env_name in ["OPENAI_API_KEY", "GROQ_API_KEY"] {
        if let Ok(value) = std::env::var(env_name) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    bail!("TTS API key is not set. Configure tts.api_key, config.api_key, TTS_API_KEY, OPENAI_API_KEY, or GROQ_API_KEY")
}

fn prepare_tts_input(text: &str, max_input_chars: usize) -> Result<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        bail!("Cannot synthesize speech for an empty reply");
    }
    if max_input_chars == 0 {
        bail!("TTS max_input_chars must be greater than 0");
    }

    Ok(truncate_with_ellipsis(trimmed, max_input_chars))
}

/// Return a file name extension suitable for the configured response format.
pub fn tts_file_name(config: &TtsConfig) -> String {
    if config.response_format.eq_ignore_ascii_case("opus") {
        "reply.ogg".to_string()
    } else {
        format!(
            "reply.{}",
            config.response_format.trim().to_ascii_lowercase()
        )
    }
}

/// Synthesize speech through an OpenAI-compatible `/v1/audio/speech` endpoint.
pub async fn synthesize_speech(
    text: &str,
    config: &TtsConfig,
    api_key_override: Option<&str>,
) -> Result<Vec<u8>> {
    #[cfg(test)]
    if let Some(result) = test_tts_override()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
    {
        return result.map_err(anyhow::Error::msg);
    }

    let input = prepare_tts_input(text, config.max_input_chars)?;
    let api_key = resolve_tts_api_key(config, api_key_override)?;
    let client = crate::config::build_runtime_proxy_client_with_timeouts("tts.openai", 30, 10);

    let response = client
        .post(&config.api_url)
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": config.model,
            "voice": config.voice,
            "input": input,
            "response_format": config.response_format,
        }))
        .send()
        .await
        .context("Failed to send TTS request")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let message = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(|error| error.get("message"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
            })
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| {
                let trimmed = body.trim();
                if trimmed.is_empty() {
                    "unknown error".to_string()
                } else {
                    trimmed.to_string()
                }
            });
        bail!("TTS API error ({status}): {message}");
    }

    let audio_bytes = response
        .bytes()
        .await
        .context("Failed to read TTS response body")?
        .to_vec();
    if audio_bytes.is_empty() {
        bail!("TTS API returned an empty audio body");
    }

    Ok(audio_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tts_config(api_url: String) -> TtsConfig {
        TtsConfig {
            enabled: true,
            api_key: None,
            api_url,
            model: "tts-1".into(),
            voice: "alloy".into(),
            response_format: "opus".into(),
            max_input_chars: 32,
            voice_reply_mode: crate::config::VoiceReplyMode::Off,
        }
    }

    #[test]
    fn tts_file_name_maps_opus_to_ogg() {
        let config = sample_tts_config("https://example.com/v1/audio/speech".into());
        assert_eq!(tts_file_name(&config), "reply.ogg");
    }

    #[test]
    fn prepare_tts_input_rejects_empty_text() {
        let error = prepare_tts_input("   ", 10).expect_err("expected empty-input failure");
        assert!(error.to_string().contains("empty reply"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn synthesize_speech_rejects_missing_api_key() {
        // SAFETY: env::remove_var is unsound in multi-threaded programs;
        // current_thread runtime ensures single-threaded execution here.
        unsafe {
            std::env::remove_var("TTS_API_KEY");
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("GROQ_API_KEY");
        }

        let error = synthesize_speech("hello", &TtsConfig::default(), None)
            .await
            .expect_err("expected missing-key failure");
        assert!(error.to_string().contains("TTS API key"));
    }

    #[tokio::test]
    async fn synthesize_speech_uses_test_override() {
        let _guard = lock_test_synthesis();
        set_test_synthesis_result(Ok(vec![1_u8, 2, 3, 4]));
        let config = sample_tts_config("https://example.invalid/v1/audio/speech".into());
        let bytes = synthesize_speech("hello world", &config, None)
            .await
            .expect("tts request should succeed");

        assert_eq!(bytes, vec![1_u8, 2, 3, 4]);
    }
}
