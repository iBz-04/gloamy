# Platform and CLI Contract

## Supported binaries

The supported native host artifacts are exactly:

- `gloamy-x86_64-unknown-linux-gnu.tar.gz` — user label `linux-amd64`.
- `gloamy-aarch64-unknown-linux-gnu.tar.gz` — user label `linux-arm64`.
- `gloamy-aarch64-apple-darwin.tar.gz` — user label `aarch64-darwin`.

The artifact names above follow the existing release packaging convention, where the actual workflow constructs `gloamy-${target}.tar.gz`.

## Installer behavior

`scripts/bootstrap.sh` recognizes only:

| `uname -s` | `uname -m` | Selected target |
| --- | --- | --- |
| `Linux` | `x86_64` | `x86_64-unknown-linux-gnu` |
| `Linux` | `aarch64` or `arm64` | `aarch64-unknown-linux-gnu` |
| `Darwin` | `arm64` or `aarch64` | `aarch64-apple-darwin` |

All other host tuples return an unsupported-target result and do not attempt a different artifact.

## Container contract

Docker publishes only `linux/amd64` and `linux/arm64`; there is no Darwin container artifact.

## Removed CLI/config contracts

Remove the `hardware` and `peripheral` command groups, the agent peripheral override, and the `[hardware]`/`[peripherals]` configuration schema. Preserve other CLI/config contracts.
