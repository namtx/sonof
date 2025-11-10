use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct LibraryBook {
    pub id: u32,
    #[serde(rename = "entityId")]
    pub entity_id: u32,
    pub entity: String,
    #[serde(rename = "purchasedType")]
    pub purchased_type: String,
    #[serde(rename = "purchasedAt")]
    pub purchased_at: String,
}

#[derive(Debug)]
pub struct BookInfo {
    pub book_id: u32,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookDetails {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub duration: f64,
    pub price: u32,
    pub status: String,
    #[serde(rename = "storageKey")]
    pub storage_key: String,
    #[serde(rename = "coverImageUrl")]
    pub cover_image_url: String,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
    #[serde(default)]
    pub categories: Vec<Category>,
    pub author: Option<serde_json::Value>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Chapter {
    pub id: u32,
    pub name: String,
    pub duration: f64,
    pub order: u32,
    pub url: String,
    #[serde(default)]
    pub voices: Vec<Voice>,
    #[serde(default)]
    pub attachments: Vec<ChapterAttachment>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Voice {
    pub url: String,
    #[serde(rename = "voiceId")]
    pub voice_id: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChapterAttachment {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
    pub id: u32,
    pub name: String,
    #[serde(rename = "showOnList")]
    pub show_on_list: bool,
    #[serde(rename = "showOnDetail")]
    pub show_on_detail: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Attachment {
    pub id: u32,
    pub name: String,
    pub url: String,
    #[serde(rename = "bookId")]
    pub book_id: u32,
    #[serde(rename = "chapterId")]
    pub chapter_id: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResourcePermissions {
    #[serde(rename = "cloud-cdn-signed-cookie")]
    pub cloud_cdn_signed_cookie: String,
    #[serde(rename = "cloud-cdn-signed-url")]
    pub cloud_cdn_signed_url: String,
    #[serde(rename = "azure-token-auth")]
    pub azure_token_auth: String,
    #[serde(rename = "vnetwork-token-auth")]
    pub vnetwork_token_auth: String,
    #[serde(rename = "byteplus-signed-token")]
    pub byteplus_signed_token: String,
    #[serde(rename = "clearKey")]
    pub clear_key: String,
}

