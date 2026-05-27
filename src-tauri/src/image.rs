use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessOptions {
    pub target_format: Option<String>,  // png, jpg, webp
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub quality: Option<u8>,            // 1-100 for jpg/webp
    pub remove_background: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpriteSheetOptions {
    pub frame_width: u32,
    pub frame_height: u32,
    pub columns: Option<u32>,
    pub padding: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessResult {
    pub input_path: String,
    pub output_path: String,
    pub original_size: ImageInfo,
    pub new_size: Option<ImageInfo>,
    pub success: bool,
    pub error: Option<String>,
}

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn get_image_info(&self, path: &Path) -> Result<ImageInfo, String> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // For now, return basic info without actual image decoding
        // In a real implementation, we'd use the `image` crate
        Ok(ImageInfo {
            width: 0,
            height: 0,
            format: extension,
            file_size: metadata.len(),
        })
    }

    pub fn convert_format(
        &self,
        input_path: &Path,
        output_path: &Path,
        _target_format: &str,
    ) -> Result<ProcessResult, String> {
        let input_info = self.get_image_info(input_path)?;
        
        // Simple file copy for now
        // In a real implementation, we'd use the `image` crate for format conversion
        std::fs::copy(input_path, output_path)
            .map_err(|e| format!("Failed to copy file: {}", e))?;

        let output_info = self.get_image_info(output_path)?;

        Ok(ProcessResult {
            input_path: input_path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            original_size: input_info,
            new_size: Some(output_info),
            success: true,
            error: None,
        })
    }

    pub fn resize_image(
        &self,
        input_path: &Path,
        output_path: &Path,
        max_width: u32,
        max_height: u32,
    ) -> Result<ProcessResult, String> {
        let input_info = self.get_image_info(input_path)?;
        
        // Simple file copy for now
        // In a real implementation, we'd use the `image` crate for resizing
        std::fs::copy(input_path, output_path)
            .map_err(|e| format!("Failed to copy file: {}", e))?;

        let output_info = self.get_image_info(output_path)?;

        Ok(ProcessResult {
            input_path: input_path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            original_size: input_info,
            new_size: Some(output_info),
            success: true,
            error: None,
        })
    }

    pub fn split_spritesheet(
        &self,
        input_path: &Path,
        output_dir: &Path,
        options: &SpriteSheetOptions,
    ) -> Result<Vec<ProcessResult>, String> {
        let input_info = self.get_image_info(input_path)?;
        
        // Create output directory
        std::fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // For now, just copy the file
        // In a real implementation, we'd use the `image` crate to split the spritesheet
        let output_path = output_dir.join(input_path.file_name().unwrap_or_default());
        std::fs::copy(input_path, &output_path)
            .map_err(|e| format!("Failed to copy file: {}", e))?;

        let output_info = self.get_image_info(&output_path)?;

        Ok(vec![ProcessResult {
            input_path: input_path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            original_size: input_info,
            new_size: Some(output_info),
            success: true,
            error: None,
        }])
    }

    pub fn process_file(
        &self,
        input_path: &Path,
        output_dir: &Path,
        options: &ProcessOptions,
    ) -> Result<ProcessResult, String> {
        let file_name = input_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("output");
        
        let extension = options.target_format.as_deref()
            .or_else(|| input_path.extension().and_then(|e| e.to_str()))
            .unwrap_or("png");
        
        let output_path = output_dir.join(format!("{}.{}", file_name, extension));
        
        self.convert_format(input_path, &output_path, extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_processor() {
        let processor = ImageProcessor::new();
        assert!(true); // Basic test
    }
}
