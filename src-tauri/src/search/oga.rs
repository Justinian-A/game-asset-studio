use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OgaAsset {
    pub id: String,
    pub title: String,
    pub author: String,
    pub url: String,
    pub preview_url: Option<String>,
    pub license: String,
    pub tags: Vec<String>,
    pub asset_type: String,
    pub source: String,
}

pub struct OgaClient {
    client: Client,
}

impl OgaClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    pub async fn search(&self, query: &str, page: u32) -> Result<Vec<OgaAsset>, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "https://opengameart.org/art-search-advanced?keys={}&field_art_type_tid=All&page={}",
            urlencoding::encode(query),
            page
        );

        log::info!("Searching OpenGameArt: {}", url);

        let response = self.client.get(&url).send().await?;
        let html = response.text().await?;
        let assets = self.parse_search_results(&html);
        
        log::info!("Found {} assets on OpenGameArt", assets.len());
        Ok(assets)
    }

    fn parse_search_results(&self, html: &str) -> Vec<OgaAsset> {
        let document = Html::parse_document(html);
        let mut assets = Vec::new();

        // Select art items
        let item_selector = Selector::parse(".art-preview, .view-content .views-row").unwrap();
        let title_selector = Selector::parse("h2 a, .field-content a").unwrap();
        let image_selector = Selector::parse("img").unwrap();
        let author_selector = Selector::parse(".username, .field-name-field-author").unwrap();

        for item in document.select(&item_selector) {
            let title_elem = item.select(&title_selector).next();
            
            let title = title_elem
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            let url = title_elem
                .and_then(|e| e.value().attr("href"))
                .map(|href| {
                    if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://opengameart.org{}", href)
                    }
                })
                .unwrap_or_default();

            let preview_url = item.select(&image_selector)
                .next()
                .and_then(|e| e.value().attr("src"))
                .map(|s| s.to_string());

            let author = item.select(&author_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // Extract ID from URL
            let id = url.split('/').last()
                .unwrap_or(&format!("oga_{}", assets.len()))
                .to_string();

            assets.push(OgaAsset {
                id,
                title,
                author,
                url,
                preview_url,
                license: "Various".to_string(),
                tags: Vec::new(),
                asset_type: "2d".to_string(),
                source: "opengameart".to_string(),
            });
        }

        assets
    }
}
