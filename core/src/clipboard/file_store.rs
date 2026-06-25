// 文件存储模块
use std::path::Path;

/// 复制文件到存储目录，保留原始扩展名
pub fn copy_file_to_store(
    files_dir: &Path,
    hash: &str,
    source_path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    // 确保存储目录存在
    std::fs::create_dir_all(files_dir)?;

    // 获取原始文件扩展名
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    // 构建目标文件名：{hash}.{ext}
    let target_name = format!("{}.{}", hash, ext);
    let target_path = files_dir.join(&target_name);

    // 复制文件
    std::fs::copy(source_path, &target_path)?;

    // 验证文件非空
    let metadata = std::fs::metadata(&target_path)?;
    if metadata.len() == 0 {
        std::fs::remove_file(&target_path)?;
        return Err("源文件为空，跳过复制".into());
    }

    Ok(target_path.to_string_lossy().to_string())
}

/// 删除存储的文件
pub fn delete_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// 格式化文件大小（B/KB/MB/GB）
pub fn format_file_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_复制文件到存储目录() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("test.txt");
        std::fs::write(&source, "hello world").unwrap();

        let result = copy_file_to_store(temp_dir.path(), "abc123", &source);
        assert!(result.is_ok());

        let target = result.unwrap();
        assert!(target.contains("abc123.txt"));
        assert!(Path::new(&target).exists());
    }

    #[test]
    fn test_删除存储文件() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.txt");
        std::fs::write(&file, "content").unwrap();

        assert!(file.exists());
        delete_file(file.to_str().unwrap()).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn test_格式化文件大小() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
