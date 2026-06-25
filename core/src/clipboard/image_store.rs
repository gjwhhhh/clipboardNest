// 图片存储模块
use image::ImageEncoder;
use std::path::Path;

pub fn save_original_png(
    images_dir: &Path,
    hash: &str,
    rgba_data: &[u8],
    width: u32,
    height: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(images_dir)?;

    let original_path = images_dir.join(format!("{}.png", hash));
    encode_rgba_png(&original_path, rgba_data, width, height)?;
    ensure_non_empty_file(&original_path, "原图")?;
    Ok(original_path.to_string_lossy().to_string())
}

pub fn save_preview_png(
    images_dir: &Path,
    hash: &str,
    rgba_data: &[u8],
    width: u32,
    height: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(images_dir)?;

    let img = image::RgbaImage::from_raw(width, height, rgba_data.to_vec())
        .ok_or("无法创建图片：数据长度与尺寸不匹配")?;

    let preview = resize_image_to_limit(&img, 1200);
    let preview_path = images_dir.join(format!("{}_preview.png", hash));
    encode_rgba_png(
        &preview_path,
        preview.as_raw(),
        preview.width(),
        preview.height(),
    )?;
    ensure_non_empty_file(&preview_path, "预览图")?;
    Ok(preview_path.to_string_lossy().to_string())
}

pub fn copy_original_image(
    images_dir: &Path,
    hash: &str,
    source_path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(images_dir)?;

    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "png".to_string());
    let original_path = images_dir.join(format!("{}.{}", hash, extension));
    std::fs::copy(source_path, &original_path)?;
    ensure_non_empty_file(&original_path, "原图")?;
    Ok(original_path.to_string_lossy().to_string())
}

pub fn save_preview_from_file(
    images_dir: &Path,
    hash: &str,
    source_path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let image = image::open(source_path)?.to_rgba8();
    save_preview_png(
        images_dir,
        hash,
        image.as_raw(),
        image.width(),
        image.height(),
    )
}

fn encode_rgba_png(
    path: &Path,
    rgba_data: &[u8],
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    let encoder = image::codecs::png::PngEncoder::new(writer);
    encoder.write_image(rgba_data, width, height, image::ExtendedColorType::Rgba8)?;
    Ok(())
}

fn ensure_non_empty_file(path: &Path, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    if std::fs::metadata(path)?.len() == 0 {
        return Err(format!("{label}保存失败：文件为空").into());
    }
    Ok(())
}

fn resize_image_to_limit(img: &image::RgbaImage, max_dimension: u32) -> image::RgbaImage {
    let (width, height) = img.dimensions();
    let longest_edge = width.max(height);
    if longest_edge <= max_dimension {
        return img.clone();
    }

    let scale = max_dimension as f64 / longest_edge as f64;
    let target_width = ((width as f64 * scale).round() as u32).max(1);
    let target_height = ((height as f64 * scale).round() as u32).max(1);

    image::imageops::resize(
        img,
        target_width,
        target_height,
        image::imageops::FilterType::Triangle,
    )
}

pub fn delete_images(
    file_path: &str,
    thumbnail_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(file_path).exists() {
        std::fs::remove_file(file_path)?;
    }

    if Path::new(thumbnail_path).exists() {
        std::fs::remove_file(thumbnail_path)?;
    }

    Ok(())
}

/// 从 PNG 文件读取图片数据，返回 (RGBA字节, 宽, 高)
pub fn load_image_as_rgba(
    file_path: &str,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let img = image::open(file_path)?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((rgba.into_raw(), w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_image(width: u32, height: u32, pixel: [u8; 4]) -> (Vec<u8>, u32, u32) {
        let rgba: Vec<u8> = (0..width * height).flat_map(|_| pixel).collect();
        (rgba, width, height)
    }

    #[test]
    fn test_保存图片原图() {
        let tmp = TempDir::new().unwrap();
        let (rgba, w, h) = create_test_image(4, 4, [255, 0, 0, 255]);

        let orig = save_original_png(tmp.path(), "testhash", &rgba, w, h).unwrap();

        assert!(Path::new(&orig).exists());
        let original = image::open(&orig).unwrap();
        assert_eq!(original.width(), 4);
        assert_eq!(original.height(), 4);
    }

    #[test]
    fn test_图片资源保存只生成原图与单张预览图() {
        let tmp = TempDir::new().unwrap();
        let (rgba, w, h) = create_test_image(2400, 1600, [0, 128, 255, 255]);

        let orig = save_original_png(tmp.path(), "bighash", &rgba, w, h).unwrap();
        let preview = save_preview_png(tmp.path(), "bighash", &rgba, w, h).unwrap();

        assert!(Path::new(&orig).exists());
        assert!(Path::new(&preview).exists());
        assert!(!tmp.path().join("bighash_200.png").exists());
        assert!(!tmp.path().join("bighash_600.png").exists());

        let preview_img = image::open(&preview).unwrap();
        assert_eq!(preview_img.width(), 1200);
        assert_eq!(preview_img.height(), 800);
    }

    #[test]
    fn test_小图预览不放大() {
        let tmp = TempDir::new().unwrap();
        let (rgba, w, h) = create_test_image(400, 200, [0, 128, 255, 255]);

        let preview = save_preview_png(tmp.path(), "smallhash", &rgba, w, h).unwrap();

        let preview_img = image::open(&preview).unwrap();
        assert_eq!(preview_img.width(), 400);
        assert_eq!(preview_img.height(), 200);
    }

    #[test]
    fn test_文件图片原图应复制源文件并保留扩展名() {
        let tmp = TempDir::new().unwrap();
        let source_path = tmp.path().join("source.JPG");
        std::fs::write(&source_path, b"source-image-bytes").unwrap();

        let original = copy_original_image(tmp.path(), "copyhash", &source_path).unwrap();

        assert!(original.ends_with("copyhash.jpg"));
        assert_eq!(std::fs::read(&original).unwrap(), b"source-image-bytes");
    }

    #[test]
    fn test_删除图片文件() {
        let tmp = TempDir::new().unwrap();
        let (rgba, w, h) = create_test_image(4, 4, [255, 0, 0, 255]);

        let orig = save_original_png(tmp.path(), "delhash", &rgba, w, h).unwrap();
        let preview = save_preview_png(tmp.path(), "delhash", &rgba, w, h).unwrap();

        assert!(Path::new(&orig).exists());
        delete_images(&orig, &preview).unwrap();

        assert!(!Path::new(&orig).exists());
        assert!(!Path::new(&preview).exists());
    }

    #[test]
    fn test_加载图片为rgba() {
        let tmp = TempDir::new().unwrap();
        let (rgba, w, h) = create_test_image(4, 4, [255, 0, 0, 255]);

        let orig = save_original_png(tmp.path(), "loadhash", &rgba, w, h).unwrap();

        let (loaded_rgba, loaded_w, loaded_h) = load_image_as_rgba(&orig).unwrap();
        assert_eq!(loaded_w, w);
        assert_eq!(loaded_h, h);
        assert_eq!(loaded_rgba.len(), (w * h * 4) as usize);
    }
}
