// 数据模型
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: i64,
    pub content_type: ContentType,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub content_hash: String,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub file_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub source_app: Option<String>,
    pub is_pinned: bool,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Richtext,
    Image,
    File,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Richtext => "richtext",
            ContentType::Image => "image",
            ContentType::File => "file",
        }
    }
}

impl FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(ContentType::Text),
            "richtext" => Ok(ContentType::Richtext),
            "image" => Ok(ContentType::Image),
            "file" => Ok(ContentType::File),
            _ => Err(format!("Unknown content type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItemCreate {
    pub content_type: ContentType,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub content_hash: String,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub file_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub source_app: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub retention_days: u32,
    pub max_items: u32,
    pub poll_interval_ms: u32,
    pub hotkey: String,
    pub launch_at_login: bool,
    pub theme: String,
}
