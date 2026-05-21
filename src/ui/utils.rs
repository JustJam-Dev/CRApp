use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn cleanup_avatar(path_str: &str) {
    let path = Path::new(path_str);
    // Security check: Only delete if inside "data/avatars"
    // Normalize logic loosely by checking components or starts_with
    if path_str.replace("\\", "/").contains("data/avatars/") {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                eprintln!("Failed to delete old avatar {}: {}", path_str, e);
            } else {
                println!("Deleted old avatar: {}", path_str);
            }
        }
        // Delete processed cache files
        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                if let Some(parent) = path.parent() {
                    let blur_path = parent.join(format!("{}_blur.{}", file_stem, extension));
                    if blur_path.exists() {
                        let _ = std::fs::remove_file(blur_path);
                    }
                    let pixel_path = parent.join(format!("{}_pixel.{}", file_stem, extension));
                    if pixel_path.exists() {
                        let _ = std::fs::remove_file(pixel_path);
                    }
                }
            }
        }
    }
}

pub fn get_processed_avatar(original_path_str: &str, mode: crate::models::BlurMode) -> Option<String> {
    if mode == crate::models::BlurMode::FullBlur {
        return Some(original_path_str.to_string());
    }

    let original_path = Path::new(original_path_str);
    if !original_path.exists() {
        return None;
    }

    let suffix = match mode {
        crate::models::BlurMode::Simple => "_blur",
        crate::models::BlurMode::Pixelize => "_pixel",
        crate::models::BlurMode::FullBlur => unreachable!(),
    };

    // Construct processed path: e.g., data/avatars/char_123_blur.png
    let file_stem = original_path.file_stem()?.to_str()?;
    let extension = original_path.extension()?.to_str()?;
    let parent = original_path.parent()?;
    let processed_file_name = format!("{}{}.{}", file_stem, suffix, extension);
    let processed_path = parent.join(processed_file_name);

    if !processed_path.exists() {
        // Lazily generate the file!
        if let Ok(img) = image::open(original_path) {
            match mode {
                crate::models::BlurMode::Simple => {
                    // Apply Gaussian blur (sigma = 15.0 looks beautifully and heavily blurred)
                    let blurred = img.blur(15.0);
                    let _ = blurred.save(&processed_path);
                }
                crate::models::BlurMode::Pixelize => {
                    // Downscale to 16x16, then upscale back to original using Nearest Neighbor filter
                    let width = img.width();
                    let height = img.height();
                    let pixel_size = 16.min(width).min(height);
                    let small = img.resize_exact(pixel_size, pixel_size, image::imageops::FilterType::Nearest);
                    let pixelated = small.resize_exact(width, height, image::imageops::FilterType::Nearest);
                    let _ = pixelated.save(&processed_path);
                }
                crate::models::BlurMode::FullBlur => {}
            }
        }
    }

    processed_path.to_str().map(|s| s.to_string())
}

pub fn get_image_uri(path: &str) -> String {
    if path.starts_with("file://") || path.contains("://") {
        return path.to_string();
    }

    // Check if absolute
    let path_obj = Path::new(path);
    if path_obj.is_absolute() {
        return format!("file://{}", path);
    }

    // Relative path: Resolve against cached current dir
    let cwd =
        CURRENT_DIR.get_or_init(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let abs = cwd.join(path);
    format!("file://{}", abs.to_string_lossy())
}
