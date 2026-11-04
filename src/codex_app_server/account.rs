use super::{
    required_string, AccountStatus, CodexAppServerClient, LoginPrompt, APP_SERVER_REQUEST_TIMEOUT,
};
use anyhow::Result;
use serde_json::{json, Value};
use std::time::Instant;

const APP_SERVER_LOGIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600);

impl CodexAppServerClient {
    /// Read the active Codex account without returning email or credential data.
    pub async fn account_status(&mut self) -> Result<AccountStatus> {
        let result = self
            .request("account/read", json!({}), APP_SERVER_REQUEST_TIMEOUT)
            .await?;
        Ok(AccountStatus {
            account_type: result
                .pointer("/account/type")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        })
    }

    /// Start a Codex-managed ChatGPT OAuth flow.
    pub async fn start_login(&mut self, use_device_code: bool) -> Result<LoginPrompt> {
        let params = if use_device_code {
            json!({ "type": "chatgptDeviceCode" })
        } else {
            json!({ "type": "chatgpt" })
        };
        let result = self
            .request("account/login/start", params, APP_SERVER_REQUEST_TIMEOUT)
            .await?;
        let login_id = required_string(&result, "loginId", "Codex App Server login response")?;

        if use_device_code {
            Ok(LoginPrompt::DeviceCode {
                login_id,
                verification_url: required_string(
                    &result,
                    "verificationUrl",
                    "Codex App Server device-code response",
                )?,
                user_code: required_string(
                    &result,
                    "userCode",
                    "Codex App Server device-code response",
                )?,
            })
        } else {
            Ok(LoginPrompt::Browser {
                login_id,
                authorization_url: required_string(
                    &result,
                    "authUrl",
                    "Codex App Server browser-login response",
                )?,
            })
        }
    }

    /// Wait for the matching App Server login completion notification.
    pub async fn wait_for_login(&mut self, login_id: &str) -> Result<()> {
        let started = Instant::now();
        loop {
            let remaining = APP_SERVER_LOGIN_TIMEOUT.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                anyhow::bail!("Timed out waiting for Codex App Server OAuth login");
            }

            let notification = self.next_notification(remaining).await?;
            if notification.get("method").and_then(Value::as_str) != Some("account/login/completed")
            {
                continue;
            }

            let Some(params) = notification.get("params") else {
                continue;
            };
            if params.get("loginId").and_then(Value::as_str) != Some(login_id) {
                continue;
            }
            if params.get("success").and_then(Value::as_bool) == Some(true) {
                let status = self.account_status().await?;
                if status.is_chatgpt_oauth() {
                    return Ok(());
                }
                anyhow::bail!(
                    "Codex App Server login completed without an active ChatGPT OAuth account"
                );
            }

            anyhow::bail!(
                "Codex App Server reported that ChatGPT OAuth login did not complete successfully"
            );
        }
    }

    /// Refresh the managed Codex account's access token without exposing it.
    pub async fn refresh_account(&mut self) -> Result<AccountStatus> {
        let result = self
            .request(
                "account/read",
                json!({ "refreshToken": true }),
                APP_SERVER_REQUEST_TIMEOUT,
            )
            .await?;
        let status = AccountStatus {
            account_type: result
                .pointer("/account/type")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        };
        if !status.is_chatgpt_oauth() {
            anyhow::bail!("No ChatGPT OAuth account is signed in through Codex App Server");
        }
        Ok(status)
    }

    /// Sign out of the account managed by Codex App Server.
    pub async fn logout(&mut self) -> Result<()> {
        self.request("account/logout", Value::Null, APP_SERVER_REQUEST_TIMEOUT)
            .await?;
        Ok(())
    }
}
