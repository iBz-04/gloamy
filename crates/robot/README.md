# Gloamy Robot

`gloamy-robot` is the optional robot-control crate in the Gloamy workspace. It owns the hardware-facing side of a mobile robot: movement, camera input, speech I/O, sensor reads, expression output, and the safety layer that can veto movement when conditions are wrong.

The point of the crate is separation. The main `gloamy` runtime stays generic; this crate handles the robot-specific edge.

## What This Crate Actually Does

It gives you one place to assemble the robot tool surface:

| Surface | Role | Current implementation |
|---|---|---|
| `drive` | Move the base | mock mode, serial controller writes, or `ros2` CLI publishing |
| `look` | Capture and inspect a scene | `ffmpeg` or `fswebcam`, optional Ollama vision call |
| `listen` | Hear speech | `arecord` plus `whisper.cpp` |
| `speak` | Talk back | Piper TTS plus `aplay` or `paplay` |
| `sense` | Read nearby state | mock scan, `rplidar_scan`, ROS2 scan echo, GPIO/sysfs checks |
| `emote` | Show simple robot state | LED FIFO or helper binary, optional sound |
| `safety` | Gate motion | `SafetyMonitor`, `SafeDrive`, preflight checks |

## Design Intent

- The crate is a workspace member, not a hidden extension bolted into the runtime.
- Hardware integrations stay explicit. If the deployment depends on `ros2`, `whisper.cpp`, `piper`, or helper binaries, the crate says so.
- Safety is outside the model loop. The model can ask to move; the safety layer decides whether movement is allowed.
- Mock mode is the default path so the software surface can be exercised before motors or sensors are live.

## What It Is Not

- It is not auto-registered into `gloamy`'s core tool factory.
- It is not a full robotics framework or ROS replacement.
- It is not pretending all backends are equally complete; some paths are helper-driven and intentionally thin.

## Build

```bash
cargo build -p gloamy-robot
```

## Configure

Start from the sample config instead of inventing one from scratch:

```bash
mkdir -p ~/.gloamy
cp crates/robot/robot.toml ~/.gloamy/robot.toml
```

The default config favors `mock` movement and conservative safety values so you can prove the control path first.

## Use From Rust

```rust
use gloamy_robot::{create_tools, RobotConfig};

fn build_robot() {
    let config = RobotConfig::default();
    let tools = create_tools(&config);

    assert_eq!(tools.len(), 6);
}
```

If you want movement to go through the safety gate:

```rust
use gloamy_robot::{create_safe_tools, RobotConfig, SafetyMonitor};
use std::sync::Arc;

fn build_safe_robot() {
    let config = RobotConfig::default();
    let (monitor, _rx) = SafetyMonitor::new(config.safety.clone());
    let tools = create_safe_tools(&config, Arc::new(monitor));

    assert_eq!(tools.len(), 6);
}
```

## Integration Model

The root binary does not expose these tools automatically. If you want them in the main runtime, add a thin adapter that maps this crate's `Tool` trait to the root tool contract and register that adapter deliberately.

That boundary is intentional. It keeps robot hardware optional, keeps permissions explicit, and avoids silently exposing a hardware surface on machines that do not have the right devices or guardrails.

## Runtime Dependencies

Depending on which surfaces you enable, a deployment may need:

- `ffmpeg` or `fswebcam`
- `ollama`
- `arecord`
- `whisper.cpp` and a local model file
- `piper` and a voice model
- `aplay` or `paplay`
- `ros2`
- helper binaries such as `rplidar_scan`, `hc-sr04`, or `gloamy-led`

## Files To Start With

- [`robot.toml`](./robot.toml): sample robot configuration
- [`PI5_SETUP.md`](./PI5_SETUP.md): Raspberry Pi 5 setup path
- [`SOUL.md`](./SOUL.md): example robot persona prompt
- [`src/lib.rs`](./src/lib.rs): crate entry point and exported surface

## Safety Rules

1. Keep `drive.backend = "mock"` until the mechanical path is verified.
2. Add a physical emergency stop before testing around people.
3. Start with conservative movement limits.
4. Treat missing helper binaries as deployment errors, not silent fallbacks.
5. Run preflight checks before any live session.

## License

MIT OR Apache-2.0
