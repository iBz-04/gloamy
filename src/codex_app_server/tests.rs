use super::*;
use std::os::unix::fs::PermissionsExt;

const FAKE_CODEX: &str = r##"#!/bin/sh
set -eu
MODE=success
while IFS= read -r line; do
    request_id=$(printf '%s\n' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
    method=$(printf '%s\n' "$line" | sed -n 's/.*"method":"\([^"]*\)".*/\1/p')
    case "$method" in
        initialize)
            if [ "$MODE" = malformed ]; then printf 'not-json\n'; exit 0; fi
            case "$line" in
                *'"experimentalApi":true'*) printf '{"id":%s,"result":{}}\n' "$request_id" ;;
                *) printf '{"id":%s,"error":{"code":-32602,"message":"experimentalApi capability required"}}\n' "$request_id" ;;
            esac
            ;;
        initialized)
            ;;
        account/read)
            if [ "$MODE" = process_exit ]; then
                exit 23
            elif [ "$MODE" = timeout ]; then
                continue
            elif [ "$MODE" = no_account ]; then
                printf '{"id":%s,"result":{"account":null}}\n' "$request_id"
            else
                printf '{"id":%s,"result":{"account":{"type":"chatgpt"}}}\n' "$request_id"
            fi
            ;;
        account/login/start)
            case "$line" in
                *chatgptDeviceCode*)
                    printf '{"id":%s,"result":{"type":"chatgptDeviceCode","loginId":"login-1","verificationUrl":"https://example.com/device","userCode":"ABCD-EFGH"}}\n' "$request_id"
                    ;;
                *)
                    printf '{"id":%s,"result":{"type":"chatgpt","loginId":"login-1","authUrl":"https://example.com/login"}}\n' "$request_id"
                    ;;
            esac
            printf '{"method":"account/login/completed","params":{"loginId":"login-1","success":true}}\n'
            ;;
        account/logout)
            printf '{"id":%s,"result":{}}\n' "$request_id"
            ;;
        model/list)
            if [ "$MODE" = rpc_error ]; then
                printf '{"id":%s,"error":{"code":-32601,"message":"private server detail"}}\n' "$request_id"
            elif [ "$MODE" = repeated_cursor ]; then
                printf '{"id":%s,"result":{"data":[],"nextCursor":"cursor-1"}}\n' "$request_id"
            else
                case "$line" in
                    *'"cursor":"cursor-1"'*)
                        printf '{"id":%s,"result":{"data":[{"id":"hidden-id","model":"hidden-model","hidden":true},{"id":"a-id","model":"model-a","hidden":false}],"nextCursor":null}}\n' "$request_id"
                        ;;
                    *)
                        printf '{"id":%s,"result":{"data":[{"id":"a-id","model":"model-a","hidden":false,"isDefault":true}],"nextCursor":"cursor-1"}}\n' "$request_id"
                        ;;
                esac
            fi
            ;;
        thread/start)
            printf '{"id":%s,"result":{"thread":{"id":"thread-1"}}}\n' "$request_id"
            ;;
        turn/start)
            printf '{"id":%s,"result":{"turn":{"id":"turn-1"}}}\n' "$request_id"
            printf '{"method":"item/agentMessage/delta","params":{"threadId":"thread-1","turnId":"turn-1","delta":"Codex "}}\n'
            printf '{"method":"item/agentMessage/delta","params":{"threadId":"thread-1","turnId":"turn-1","delta":"response"}}\n'
            if [ "$MODE" = failed_turn ]; then
                printf '{"method":"turn/completed","params":{"threadId":"thread-1","turn":{"id":"turn-1","status":"failed"}}}\n'
            else
                printf '{"method":"turn/completed","params":{"threadId":"thread-1","turn":{"id":"turn-1","status":"completed"}}}\n'
            fi
            ;;
        *)
            printf '{"id":%s,"result":{}}\n' "$request_id"
            ;;
    esac
done
"##;

fn fake_codex(mode: &str) -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("temporary test directory should be created");
    let path = directory.path().join("codex");
    let script = FAKE_CODEX.replace("MODE=success", &format!("MODE={mode}"));
    std::fs::write(&path, script).expect("fake Codex executable should be written");
    let mut permissions = std::fs::metadata(&path)
        .expect("fake Codex executable should exist")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&path, permissions)
        .expect("fake Codex executable should be executable");
    (directory, path)
}

fn require_error<T>(result: Result<T>, context: &str) -> anyhow::Error {
    match result {
        Ok(_) => panic!("{context}"),
        Err(error) => error,
    }
}

#[tokio::test]
async fn app_server_manages_login_account_and_logout() {
    let (_directory, binary) = fake_codex("success");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");

    let prompt = app_server
        .start_login(true)
        .await
        .expect("device-code login should start");
    let LoginPrompt::DeviceCode {
        login_id,
        verification_url,
        user_code,
    } = prompt
    else {
        panic!("device-code request should return a device-code prompt");
    };
    assert_eq!(login_id, "login-1");
    assert_eq!(verification_url, "https://example.com/device");
    assert_eq!(user_code, "ABCD-EFGH");
    app_server
        .wait_for_login(&login_id)
        .await
        .expect("matching login notification should complete login");
    assert!(app_server
        .account_status()
        .await
        .expect("account status should be readable")
        .is_chatgpt_oauth());
    app_server
        .refresh_account()
        .await
        .expect("managed account refresh should be accepted");
    app_server
        .logout()
        .await
        .expect("managed account logout should be accepted");
}

#[tokio::test]
async fn model_discovery_reads_all_visible_pages_and_preserves_ids() {
    let (_directory, binary) = fake_codex("success");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");
    let models = app_server
        .list_models()
        .await
        .expect("model catalog should load");

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "model-a");
    assert!(models[0].is_default);
}

#[tokio::test]
async fn inference_uses_streamed_deltas_and_requires_successful_turn() {
    let (_directory, binary) = fake_codex("success");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");
    let result = app_server
        .infer(
            "model-a",
            "test instructions",
            vec![json!({ "type": "text", "text": "test input" })],
        )
        .await
        .expect("completed turn should return its deltas");
    assert_eq!(result, "Codex response");

    let (_directory, binary) = fake_codex("failed_turn");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");
    let error = require_error(
        app_server
            .infer(
                "model-a",
                "test instructions",
                vec![json!({ "type": "text", "text": "test input" })],
            )
            .await,
        "failed turn must remain an error",
    );
    assert!(error.to_string().contains("status `failed`"));
}

#[tokio::test]
async fn model_discovery_rejects_missing_auth_repeated_cursor_and_rpc_error() {
    let (_directory, binary) = fake_codex("no_account");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");
    let error = require_error(
        app_server.list_models().await,
        "signed-out app-server must not expose a selectable catalog",
    );
    assert!(error.to_string().contains("No ChatGPT OAuth account"));

    let (_directory, binary) = fake_codex("repeated_cursor");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");
    let error = require_error(
        app_server.list_models().await,
        "repeated pagination cursor must fail",
    );
    assert!(error.to_string().contains("repeated a pagination cursor"));

    let (_directory, binary) = fake_codex("rpc_error");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");
    let error = require_error(
        app_server.list_models().await,
        "RPC failure must not be treated as an empty model list",
    );
    assert!(error.to_string().contains("JSON-RPC -32601"));
    assert!(!error.to_string().contains("private server detail"));
}

#[tokio::test]
async fn invalid_app_server_output_fails_without_echoing_payload() {
    let (_directory, binary) = fake_codex("malformed");
    let error = require_error(
        CodexAppServerClient::connect_with_binary(binary).await,
        "malformed protocol output must fail initialization",
    );
    let error_chain = format!("{error:#}");
    assert!(error_chain.contains("invalid JSON-RPC message"));
    assert!(!error_chain.contains("not-json"));
}

#[tokio::test]
async fn app_server_process_exit_is_reported_with_status() {
    let (_directory, binary) = fake_codex("process_exit");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");
    let error = require_error(
        app_server.account_status().await,
        "unexpected App Server exit must fail the account request",
    );
    let error_chain = format!("{error:#}");
    assert!(error_chain.contains("closed its stdio stream"));
    assert!(error_chain.contains("process status: Some("));
}

#[tokio::test]
async fn app_server_request_timeout_is_reported() {
    let (_directory, binary) = fake_codex("timeout");
    let mut app_server = CodexAppServerClient::connect_with_binary(binary)
        .await
        .expect("fake App Server should initialize");
    let error = require_error(
        app_server
            .request("account/read", json!({}), Duration::from_millis(25))
            .await,
        "unanswered App Server request must time out",
    );
    assert!(error
        .to_string()
        .contains("timed out waiting for Codex App Server `account/read` response"));
}

#[test]
fn inference_thread_is_ephemeral_and_has_no_external_tool_roots() {
    let cwd = PathBuf::from("/tmp/gloamy-codex-test");
    let params =
        super::inference::restricted_thread_start_params(" model-a ", "instructions", &cwd);

    assert_eq!(params["model"], "model-a");
    assert_eq!(params["ephemeral"], true);
    assert_eq!(params["sandbox"], "read-only");
    assert_eq!(params["approvalPolicy"], "never");
    assert_eq!(params["cwd"], "/tmp/gloamy-codex-test");
    assert_eq!(params["runtimeWorkspaceRoots"][0], "/tmp/gloamy-codex-test");
    assert_eq!(params["environments"], json!([]));
    assert_eq!(params["dynamicTools"], json!([]));
    assert_eq!(params["selectedCapabilityRoots"], json!([]));
    assert_eq!(params["config"]["mcp_servers"], json!({}));
    assert_eq!(params["config"]["web_search"], "disabled");
    assert!(
        super::inference::restricted_thread_start_params("default", "instructions", &cwd)
            .get("model")
            .is_none()
    );
}
