//! # Gloamy Robot
//!
//! `gloamy-robot` is the standalone robot-control crate for Gloamy. It groups
//! the robot-facing tool surface in one library so robot integrations can
//! evolve without coupling every hardware concern into the root runtime.
//!
//! ## Included tool surfaces
//!
//! - [`DriveTool`]: movement commands for mock, serial, or ROS2-backed bases
//! - [`LookTool`]: image capture plus optional Ollama-based vision description
//! - [`ListenTool`]: offline speech-to-text using `arecord` and `whisper.cpp`
//! - [`SpeakTool`]: Piper-based text-to-speech and canned sound playback
//! - [`SenseTool`]: obstacle, distance, and motion reads
//! - [`EmoteTool`]: lightweight LED and sound-driven expression output
//! - [`SafetyMonitor`] and [`SafeDrive`]: movement gating and emergency-stop support
//!
//! ## Integration model
//!
//! This crate is a workspace member, but it is not auto-registered into the
//! main `gloamy` tool factory. The expected pattern today is:
//!
//! 1. load or construct a [`RobotConfig`]
//! 2. build tools with [`create_tools`] or [`create_safe_tools`]
//! 3. optionally adapt this crate's [`Tool`] trait into the main runtime
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use gloamy_robot::{create_tools, RobotConfig};
//!
//! let config = RobotConfig::default();
//! let tools = create_tools(&config);
//!
//! assert_eq!(tools.len(), 6);
//! ```
//!
//! ## Safety model
//!
//! The control plane may request movement, but [`SafetyMonitor`] decides whether
//! movement is allowed. That separation is intentional and keeps collision
//! handling, watchdog behavior, and emergency-stop responses outside the LLM's
//! decision loop.

// TODO: Re-enable once the public API surface is documented end-to-end.
// #![warn(missing_docs)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod traits;

pub mod drive;
pub mod emote;
pub mod listen;
pub mod look;
pub mod sense;
pub mod speak;

#[cfg(feature = "safety")]
pub mod safety;

#[cfg(test)]
mod tests;

// Re-exports for convenience
pub use config::RobotConfig;
pub use traits::{Tool, ToolResult, ToolSpec};

pub use drive::DriveTool;
pub use emote::EmoteTool;
pub use listen::ListenTool;
pub use look::LookTool;
pub use sense::SenseTool;
pub use speak::SpeakTool;

#[cfg(feature = "safety")]
pub use safety::{preflight_check, SafeDrive, SafetyEvent, SafetyMonitor, SensorReading};

/// Version of the published crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build the standard robot tool set from a shared configuration.
///
/// The returned tools are standalone and do not wrap `drive` with a safety gate.
pub fn create_tools(config: &RobotConfig) -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(DriveTool::new(config.clone())),
        Box::new(LookTool::new(config.clone())),
        Box::new(ListenTool::new(config.clone())),
        Box::new(SpeakTool::new(config.clone())),
        Box::new(SenseTool::new(config.clone())),
        Box::new(EmoteTool::new(config.clone())),
    ]
}

/// Build the standard robot tool set with safety-enforced drive control.
///
/// This keeps the visible tool list the same while routing movement requests
/// through [`SafeDrive`].
#[cfg(feature = "safety")]
pub fn create_safe_tools(
    config: &RobotConfig,
    safety: std::sync::Arc<SafetyMonitor>,
) -> Vec<Box<dyn Tool>> {
    let drive = std::sync::Arc::new(DriveTool::new(config.clone()));
    let safe_drive = SafeDrive::new(drive, safety);

    vec![
        Box::new(safe_drive),
        Box::new(LookTool::new(config.clone())),
        Box::new(ListenTool::new(config.clone())),
        Box::new(SpeakTool::new(config.clone())),
        Box::new(SenseTool::new(config.clone())),
        Box::new(EmoteTool::new(config.clone())),
    ]
}
