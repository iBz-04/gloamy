use crate::perception::traits::{OcrTextItem, ScreenBounds};
use anyhow::{Context, Result};
use std::path::Path;
use tokio::process::Command;

/// Deterministic OCR settings applied to each Tesseract invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrConfig {
    /// Tesseract language code(s), for example `eng` or `eng+deu`.
    pub language: String,
    /// Page segmentation mode (0..=13).
    pub psm: u8,
    /// OCR engine mode (0..=3).
    pub oem: u8,
    /// Optional tessdata directory to pin traineddata source.
    pub tessdata_dir: Option<String>,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            psm: 11,
            oem: 1,
            tessdata_dir: None,
        }
    }
}

impl OcrConfig {
    pub fn validate(&self) -> Result<()> {
        let language = self.language.trim();
        if language.is_empty() {
            anyhow::bail!("OCR language cannot be empty");
        }
        if language.len() > 64 {
            anyhow::bail!("OCR language is too long");
        }
        if !language
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '_' | '-' | '/' | '.'))
        {
            anyhow::bail!("OCR language contains unsupported characters");
        }

        if self.psm > 13 {
            anyhow::bail!("OCR psm must be in range 0..=13");
        }
        if self.oem > 3 {
            anyhow::bail!("OCR oem must be in range 0..=3");
        }

        if self
            .tessdata_dir
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            anyhow::bail!("OCR tessdata_dir cannot be empty when provided");
        }

        Ok(())
    }
}

/// Uses Tesseract CLI to extract text and bounding boxes from an image.
/// Requires `tesseract` to be installed on the host system.
pub struct TesseractOcrProvider;

impl TesseractOcrProvider {
    pub async fn extract_text(image_path: &Path) -> Result<Vec<OcrTextItem>> {
        Self::extract_text_with_config(image_path, &OcrConfig::default()).await
    }

    pub async fn extract_text_with_config(
        image_path: &Path,
        config: &OcrConfig,
    ) -> Result<Vec<OcrTextItem>> {
        config.validate()?;
        let image_str = image_path
            .to_str()
            .context("Invalid image path for Tesseract")?;

        // `tesseract input stdout ... tsv quiet` produces TSV output with
        // confidence and bounding boxes, while suppressing non-essential logs.
        let mut command = Command::new("tesseract");
        command
            .arg(image_str)
            .arg("stdout")
            .arg("--oem")
            .arg(config.oem.to_string())
            .arg("--psm")
            .arg(config.psm.to_string())
            .arg("-l")
            .arg(config.language.trim());

        if let Some(tessdata_dir) = config.tessdata_dir.as_deref() {
            command.arg("--tessdata-dir").arg(tessdata_dir.trim());
        }

        let output = command
            .arg("tsv")
            .arg("quiet")
            .output()
            .await
            .context("Failed to execute tesseract command. Is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let tessdata_hint = config
                .tessdata_dir
                .as_deref()
                .map(|dir| format!(", tessdata_dir={dir}"))
                .unwrap_or_default();
            anyhow::bail!(
                "Tesseract OCR failed (language={}, psm={}, oem={}{}): {}",
                config.language.trim(),
                config.psm,
                config.oem,
                tessdata_hint,
                stderr
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let items = Self::parse_tsv(&stdout);

        Ok(items)
    }

    fn parse_tsv(tsv_content: &str) -> Vec<OcrTextItem> {
        let mut items = Vec::new();
        // Skip header line
        let mut lines = tsv_content.lines();
        if let Some(header) = lines.next() {
            if !header.starts_with("level") {
                // Not the expected format, let's just attempt to parse anyway
            }
        }

        for line in lines {
            let parts: Vec<&str> = line.split('\t').collect();
            // Expected length: 12 fields
            // 0: level, 1: page_num, 2: block_num, 3: par_num, 4: line_num, 5: word_num
            // 6: left, 7: top, 8: width, 9: height, 10: conf, 11: text
            if parts.len() < 12 {
                continue;
            }

            let text = parts[11].trim();
            if text.is_empty() {
                continue;
            }

            // Parse confidence
            let conf: f64 = parts[10].parse().unwrap_or(0.0);
            if conf < 30.0 {
                // Ignore very low confidence results to reduce noise
                continue;
            }

            // Parse bounding box
            let left: i64 = parts[6].parse().unwrap_or(0);
            let top: i64 = parts[7].parse().unwrap_or(0);
            let width: i64 = parts[8].parse().unwrap_or(0);
            let height: i64 = parts[9].parse().unwrap_or(0);

            items.push(OcrTextItem {
                text: text.to_string(),
                bounds: ScreenBounds {
                    x: left,
                    y: top,
                    width,
                    height,
                },
                confidence: conf,
            });
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use super::{OcrConfig, TesseractOcrProvider};

    #[test]
    fn parse_tsv_extracts_high_confidence_items() {
        let tsv = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t10\t20\t30\t40\t95.5\tOpen
5\t1\t1\t1\t1\t2\t50\t60\t70\t80\t12.0\tNoise
";

        let items = TesseractOcrProvider::parse_tsv(tsv);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, "Open");
        assert_eq!(items[0].bounds.x, 10);
        assert_eq!(items[0].bounds.y, 20);
        assert_eq!(items[0].bounds.width, 30);
        assert_eq!(items[0].bounds.height, 40);
    }

    #[test]
    fn parse_tsv_ignores_malformed_rows() {
        let tsv = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
invalid\trow
5\t1\t1\t1\t1\t1\t0\t0\t10\t10\t88\tOK
";

        let items = TesseractOcrProvider::parse_tsv(tsv);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, "OK");
    }

    #[test]
    fn ocr_config_defaults_are_pinned() {
        let config = OcrConfig::default();
        assert_eq!(config.language, "eng");
        assert_eq!(config.psm, 11);
        assert_eq!(config.oem, 1);
        assert!(config.tessdata_dir.is_none());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn ocr_config_rejects_invalid_ranges() {
        let invalid_psm = OcrConfig {
            psm: 99,
            ..OcrConfig::default()
        };
        let invalid_oem = OcrConfig {
            oem: 99,
            ..OcrConfig::default()
        };
        assert!(invalid_psm.validate().is_err());
        assert!(invalid_oem.validate().is_err());
    }

    #[test]
    fn ocr_config_rejects_invalid_language() {
        let config = OcrConfig {
            language: "en g".to_string(),
            ..OcrConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
