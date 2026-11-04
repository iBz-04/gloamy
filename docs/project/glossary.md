# Gloamy Domain Glossary

This glossary defines terms used in provider and authentication documentation.

| Term | Meaning |
|---|---|
| **Codex App Server** | The local JSON-RPC service launched by `codex app-server`. Gloamy uses it through the separate `codex-app-server` provider. |
| **Codex-managed account** | The account and OAuth credentials owned by the Codex CLI under its effective `CODEX_HOME`. Gloamy invokes account operations but does not read or persist its tokens. |
| **Gloamy auth profile** | A credential profile stored by Gloamy for providers such as `openai-codex`. It is independent of the Codex CLI account. |
| **Model catalog** | Model IDs returned by a provider's discovery interface. Listing a model does not guarantee account entitlement or successful inference. |
| **Ephemeral thread** | A Codex App Server thread that is not materialized as durable conversation history. |
| **Prompt-guided tool calling** | Gloamy's text protocol for asking the model to request a Gloamy tool. The model does not receive direct authority to execute that tool. |
