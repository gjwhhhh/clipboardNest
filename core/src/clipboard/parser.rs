// 内容解析模块
use crate::storage::models::ContentType;

pub fn detect_content_type(text: &str) -> ContentType {
    // 简单启发式：如果看起来像富文本，标记为 richtext
    if text.starts_with("{\\rtf") {
        return ContentType::Richtext;
    }
    ContentType::Text
}

pub fn detect_content_type_with_types(type_identifiers: &[&str], _data: &[u8]) -> ContentType {
    for type_id in type_identifiers {
        match *type_id {
            "public.rtf" | "com.apple.flat-rtfd" => return ContentType::Richtext,
            "public.png"
            | "public.jpeg"
            | "public.tiff"
            | "com.compuserve.gif"
            | "org.webmproject.webp" => {
                return ContentType::Image;
            }
            "public.file-url" | "com.apple.pasteboard.filenames" => return ContentType::File,
            _ => {}
        }
    }
    ContentType::Text
}

pub fn generate_preview(text: &str, max_len: usize) -> String {
    // 清理空白字符用于预览
    let cleaned: String = text.split_whitespace().collect::<Vec<&str>>().join(" ");

    if cleaned.chars().count() <= max_len {
        cleaned
    } else {
        let truncated: String = cleaned.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// 将 RTF 内容降级为纯文本
pub fn rtf_to_plain_text(rtf: &str) -> String {
    // 简单的 RTF 解析：移除所有 RTF 标记，保留纯文本
    let mut result = String::new();
    let mut in_control_word = false;
    let mut in_group: i32 = 0;

    for ch in rtf.chars() {
        match ch {
            '\\' => {
                in_control_word = true;
            }
            '{' => {
                in_group += 1;
                in_control_word = false;
            }
            '}' => {
                in_group = in_group.saturating_sub(1);
                in_control_word = false;
            }
            ' ' if in_control_word => {
                in_control_word = false;
            }
            _ if in_control_word && ch.is_alphanumeric() => {
                // 跳过控制字
            }
            _ if !in_control_word && in_group > 0 => {
                result.push(ch);
            }
            _ => {}
        }
    }

    // 清理多余空白
    result.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// 检测内容类型，支持 RTF 降级
pub fn detect_and_normalize_content(text: &str) -> (ContentType, String) {
    if text.starts_with("{\\rtf") {
        let plain_text = rtf_to_plain_text(text);
        (ContentType::Richtext, plain_text)
    } else {
        (ContentType::Text, text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_检测文本内容() {
        let content = detect_content_type("你好，世界！");
        assert_eq!(content, ContentType::Text);
    }

    #[test]
    fn test_检测富文本内容() {
        let content = detect_content_type_with_types(&["public.rtf"], b"{\\rtf1\\ansi hello}");
        assert_eq!(content, ContentType::Richtext);
    }

    #[test]
    fn test_检测图片内容() {
        let content = detect_content_type_with_types(&["public.png"], &[0x89, 0x50, 0x4E, 0x47]);
        assert_eq!(content, ContentType::Image);
    }

    #[test]
    fn test_生成文本预览() {
        let preview = generate_preview("你好，世界！这是一段很长的文本，应该被截断。", 10);
        assert!(preview.chars().count() <= 10);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_生成短文本预览() {
        let preview = generate_preview("短文本", 20);
        assert_eq!(preview, "短文本");
    }

    #[test]
    fn test_rtf转纯文本() {
        let rtf = "{\\rtf1\\ansi This is \\b bold \\b0 text.}";
        let result = rtf_to_plain_text(rtf);
        assert!(result.contains("This is"));
        assert!(result.contains("bold"));
        assert!(result.contains("text."));
        assert!(!result.contains("\\rtf"));
        assert!(!result.contains("\\b"));
    }

    #[test]
    fn test_检测富文本内容类型() {
        let rtf = "{\\rtf1\\ansi hello}";
        let (content_type, text) = detect_and_normalize_content(rtf);
        assert_eq!(content_type, ContentType::Richtext);
        assert!(text.contains("hello"));
    }

    #[test]
    fn test_普通文本保持不变() {
        let text = "Hello, world!";
        let (content_type, result) = detect_and_normalize_content(text);
        assert_eq!(content_type, ContentType::Text);
        assert_eq!(result, text);
    }
}
