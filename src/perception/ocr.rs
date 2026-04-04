use crate::perception::traits::{OcrTextItem, ScreenBounds};
use anyhow::{Context, Result};
use std::path::Path;
use tokio::process::Command;

/// Uses Tesseract CLI to extract text and bounding boxes from an image.
/// Requires `tesseract` to be installed on the host system.
pub struct TesseractOcrProvider;

impl TesseractOcrProvider {
    pub async fn extract_text(image_path: &Path) -> Result<Vec<OcrTextItem>> {
        let image_str = image_path
            .to_str()
            .context("Invalid image path for Tesseract")?;

        // `tesseract input stdout tsv` produces TSV output containing bounding boxes and confidence.
        let output = Command::new("tesseract")
            .arg(image_str)
            .arg("stdout")
            .arg("tsv")
            .output()
            .await
            .context("Failed to execute tesseract command. Is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Tesseract OCR failed: {}", stderr);
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
    use super::TesseractOcrProvider;

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
}
