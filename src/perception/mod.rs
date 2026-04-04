pub mod macos;
pub mod ocr;
pub mod traits;

pub use macos::MacOsPerceptionProvider;
pub use ocr::{OcrConfig, TesseractOcrProvider};
