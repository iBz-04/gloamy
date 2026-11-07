# Data Model and Contracts: Supported Host Architecture Prune

This feature does not add persisted domain data. It removes product support declarations and board-specific configuration.

## Supported Host Architecture

| User label | OS | CPU architecture | Rust target | Artifact suffix |
| --- | --- | --- | --- | --- |
| `linux-amd64` | Linux | x86_64 | `x86_64-unknown-linux-gnu` | `x86_64-unknown-linux-gnu` |
| `linux-arm64` | Linux | aarch64 | `aarch64-unknown-linux-gnu` | `aarch64-unknown-linux-gnu` |
| `aarch64-darwin` | Darwin/macOS | aarch64 | `aarch64-apple-darwin` | `aarch64-apple-darwin` |

Host detection must map only the exact OS/architecture pairs above. Unsupported pairs must not fall back to another artifact.

## Removed Configuration Surfaces

- `[hardware]`: wizard-driven native GPIO, serial, probe, and datasheet settings.
- `[peripherals]`: board list, transport, serial path, baud, and datasheet directory.
- `gloamy --peripheral <board:path>` / agent peripheral override.

After removal, the existing config loader's unknown-key behavior is the compatibility path for these old keys. The loader must continue to warn about ignored fields if it already does so; no hardware migration is created.

## Removed SOP Surface

- `SopTrigger::Peripheral` and `SopTriggerSource::Peripheral`.
- Peripheral board/signal dispatch helper.

Other SOP sources (MQTT, webhook, cron, manual) remain unchanged.
