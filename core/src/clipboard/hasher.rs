// 哈希计算模块
use sha2::{Digest, Sha256};

pub fn hash_text(text: &str) -> String {
    hash_bytes(text.as_bytes())
}

pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

// 简单的十六进制编码，无需外部依赖
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_哈希文本内容() {
        let hash1 = hash_text("你好，世界！");
        let hash2 = hash_text("你好，世界！");
        let hash3 = hash_text("不同的文本");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_哈希字节数据() {
        let data = vec![1, 2, 3, 4, 5];
        let hash = hash_bytes(&data);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 十六进制长度
    }
}
