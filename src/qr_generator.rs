use qrcode::{QrCode, EcLevel, Color};
use image::{ImageBuffer, Rgb, RgbImage};
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;

#[derive(Clone)]
pub struct QrGenerator;

impl QrGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a QR code and return it as a base64-encoded PNG image
    pub fn generate_qr_base64(&self, data: &str) -> Result<String, QrGeneratorError> {
        let qr_code = QrCode::with_error_correction_level(data, EcLevel::M)
            .map_err(|e| QrGeneratorError::QrCodeGeneration(e.to_string()))?;

        // Create an image from the QR code
        let image = self.qr_code_to_image(&qr_code)?;

        // Convert image to PNG bytes
        let mut png_bytes = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_bytes);
            image.write_to(&mut cursor, image::ImageOutputFormat::Png)
                .map_err(|e| QrGeneratorError::ImageEncoding(e.to_string()))?;
        }

        // Encode as base64
        let base64_string = general_purpose::STANDARD.encode(&png_bytes);
        Ok(base64_string)
    }

    /// Generate a QR code and return it as PNG bytes
    pub fn generate_qr_png(&self, data: &str) -> Result<Vec<u8>, QrGeneratorError> {
        let qr_code = QrCode::with_error_correction_level(data, EcLevel::M)
            .map_err(|e| QrGeneratorError::QrCodeGeneration(e.to_string()))?;

        let image = self.qr_code_to_image(&qr_code)?;

        let mut png_bytes = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_bytes);
            image.write_to(&mut cursor, image::ImageOutputFormat::Png)
                .map_err(|e| QrGeneratorError::ImageEncoding(e.to_string()))?;
        }

        Ok(png_bytes)
    }

    fn qr_code_to_image(&self, qr_code: &QrCode) -> Result<RgbImage, QrGeneratorError> {
        let modules = qr_code.to_colors();
        let width = qr_code.width();
        let scale = 4; // Scale factor to make QR code smaller for better page density
        let border = 2 * scale; // Smaller border around QR code

        let img_width = (width * scale) + (border * 2);
        let img_height = img_width;

        let mut image: RgbImage = ImageBuffer::new(img_width as u32, img_height as u32);

        // Fill with white background
        for pixel in image.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Draw QR code modules
        for (y, row) in modules.chunks(width).enumerate() {
            for (x, &module) in row.iter().enumerate() {
                let color = match module {
                    Color::Dark => Rgb([0, 0, 0]),    // Black for dark modules
                    Color::Light => Rgb([255, 255, 255]), // White for light modules
                };

                // Draw scaled module
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = (x * scale + dx + border) as u32;
                        let py = (y * scale + dy + border) as u32;
                        
                        if px < img_width as u32 && py < img_height as u32 {
                            image.put_pixel(px, py, color);
                        }
                    }
                }
            }
        }

        Ok(image)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QrGeneratorError {
    #[error("QR code generation failed: {0}")]
    QrCodeGeneration(String),
    
    #[error("Image encoding failed: {0}")]
    ImageEncoding(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_base64() {
        let generator = QrGenerator::new();
        let result = generator.generate_qr_base64("TEST");
        assert!(result.is_ok());
        
        let base64 = result.unwrap();
        assert!(!base64.is_empty());
        
        // Check that it's valid base64
        assert!(general_purpose::STANDARD.decode(&base64).is_ok());
    }

    #[test]
    fn test_generate_wifi_qr() {
        let generator = QrGenerator::new();
        let wifi_data = "WIFI:T:WPA;S:TestNetwork;P:password123;H:false;;";
        let result = generator.generate_qr_base64(wifi_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_qr_png() {
        let generator = QrGenerator::new();
        let result = generator.generate_qr_png("TEST");
        assert!(result.is_ok());
        
        let png_bytes = result.unwrap();
        assert!(!png_bytes.is_empty());
        
        // Check PNG signature
        assert_eq!(&png_bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}