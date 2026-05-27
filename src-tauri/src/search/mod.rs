use serde::{Deserialize, Serialize};

pub mod itch;
pub mod oga;
pub mod kenney;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub id: String,
    pub title: String,
    pub author: String,
    pub url: String,
    pub preview_url: Option<String>,
    pub price: Option<String>,
    pub license: String,
    pub short_text: String,
    pub tags: Vec<String>,
    pub asset_type: String,
    pub source: String,
}

impl From<itch::ItchAsset> for Asset {
    fn from(a: itch::ItchAsset) -> Self {
        Self {
            id: format!("itch_{}", a.id),
            title: a.title,
            author: a.author,
            url: a.url,
            preview_url: a.cover_url,
            price: Some(a.price),
            license: "Various".to_string(),
            short_text: a.short_text,
            tags: a.tags,
            asset_type: "2d".to_string(),
            source: "itch.io".to_string(),
        }
    }
}

impl From<oga::OgaAsset> for Asset {
    fn from(a: oga::OgaAsset) -> Self {
        Self {
            id: format!("oga_{}", a.id),
            title: a.title,
            author: a.author,
            url: a.url,
            preview_url: a.preview_url,
            price: Some("Free".to_string()),
            license: a.license,
            short_text: String::new(),
            tags: a.tags,
            asset_type: a.asset_type,
            source: "opengameart".to_string(),
        }
    }
}

impl From<kenney::KenneyAsset> for Asset {
    fn from(a: kenney::KenneyAsset) -> Self {
        Self {
            id: format!("kenney_{}", a.id),
            title: a.title,
            author: "Kenney".to_string(),
            url: a.url,
            preview_url: a.preview_url,
            price: Some("Free".to_string()),
            license: a.license,
            short_text: String::new(),
            tags: a.tags,
            asset_type: a.asset_type,
            source: "kenney".to_string(),
        }
    }
}

pub struct SearchEngine {
    itch_client: itch::ItchClient,
    oga_client: oga::OgaClient,
    kenney_client: kenney::KenneyClient,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            itch_client: itch::ItchClient::new(),
            oga_client: oga::OgaClient::new(),
            kenney_client: kenney::KenneyClient::new(),
        }
    }

    pub async fn search_all(&self, query: &str, sources: Vec<String>) -> Vec<Asset> {
        let mut all_assets = Vec::new();
        
        // Search in parallel
        let mut handles = Vec::new();

        if sources.contains(&"itch.io".to_string()) || sources.is_empty() {
            let query = query.to_string();
            let itch_client = itch::ItchClient::new();
            handles.push(tokio::spawn(async move {
                itch_client.search(&query, 1).await.unwrap_or_default()
                    .into_iter().map(Asset::from).collect::<Vec<_>>()
            }));
        }

        if sources.contains(&"opengameart".to_string()) || sources.is_empty() {
            let query = query.to_string();
            let oga_client = oga::OgaClient::new();
            handles.push(tokio::spawn(async move {
                oga_client.search(&query, 1).await.unwrap_or_default()
                    .into_iter().map(Asset::from).collect::<Vec<_>>()
            }));
        }

        if sources.contains(&"kenney".to_string()) || sources.is_empty() {
            let query = query.to_string();
            let kenney_client = kenney::KenneyClient::new();
            handles.push(tokio::spawn(async move {
                kenney_client.search(&query, 1).await.unwrap_or_default()
                    .into_iter().map(Asset::from).collect::<Vec<_>>()
            }));
        }

        for handle in handles {
            if let Ok(assets) = handle.await {
                all_assets.extend(assets);
            }
        }

        all_assets
    }

    pub async fn search_source(&self, query: &str, source: &str, page: u32) -> Result<Vec<Asset>, String> {
        match source {
            "itch.io" => {
                self.itch_client.search(query, page).await
                    .map(|assets| assets.into_iter().map(Asset::from).collect())
                    .map_err(|e| e.to_string())
            }
            "opengameart" => {
                self.oga_client.search(query, page).await
                    .map(|assets| assets.into_iter().map(Asset::from).collect())
                    .map_err(|e| e.to_string())
            }
            "kenney" => {
                self.kenney_client.search(query, page).await
                    .map(|assets| assets.into_iter().map(Asset::from).collect())
                    .map_err(|e| e.to_string())
            }
            _ => Err(format!("Unknown source: {}", source))
        }
    }
}
