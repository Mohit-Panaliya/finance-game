use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct PageMeta {
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
    pub total_items: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    #[serde(flatten)]
    pub meta: PageMeta,
}

#[derive(Serialize, Deserialize)]
pub struct OkResponse {
    pub ok: bool,
}

pub fn ok() -> serde_json::Value {
    serde_json::json!({ "ok": true })
}
