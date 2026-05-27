use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItchAsset {
    pub id: String,
    pub title: String,
    pub author: String,
    pub url: String,
    pub cover_url: Option<String>,
    pub price: String,
    pub short_text: String,
    pub platforms: Vec<String>,
    pub tags: Vec<String>,
    pub source: String,
}

pub struct ItchClient {
    client: Client,
}

impl ItchClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    pub async fn search(&self, query: &str, page: u32) -> Result<Vec<ItchAsset>, Box<dyn Error + Send + Sync>> {
        // itch.io 搜索不支持 type 参数，使用通用搜索
        let url = if page > 1 {
            format!(
                "https://itch.io/search?q={}&page={}",
                urlencoding::encode(query),
                page
            )
        } else {
            format!(
                "https://itch.io/search?q={}",
                urlencoding::encode(query)
            )
        };

        log::info!("Searching itch.io: {}", url);

        let response = self.client.get(&url)
            .send()
            .await?;
        
        let html = response.text().await?;
        let assets = self.parse_search_results(&html);
        
        log::info!("Found {} assets on itch.io", assets.len());
        Ok(assets)
    }

    fn parse_search_results(&self, html: &str) -> Vec<ItchAsset> {
        let document = Html::parse_document(html);
        let mut assets = Vec::new();

        // itch.io 使用 game_cell 类
        let cell_selector = Selector::parse("[data-game_id]").unwrap();
        let title_selector = Selector::parse(".game_title a.title").unwrap();
        let author_selector = Selector::parse(".game_author a").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        let price_selector = Selector::parse(".price_value").unwrap();
        let text_selector = Selector::parse(".game_text").unwrap();

        for cell in document.select(&cell_selector) {
            // 提取标题
            let title = cell.select(&title_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            // 提取URL
            let url = cell.select(&title_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .unwrap_or("")
                .to_string();

            // 提取作者
            let author = cell.select(&author_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // 提取封面图 - 优先使用 data-lazy_src
            let cover_url = cell.select(&img_selector)
                .next()
                .and_then(|e| {
                    e.value().attr("data-lazy_src")
                        .or(e.value().attr("src"))
                        .filter(|s| !s.is_empty() && !s.contains("placeholder") && s.starts_with("http"))
                        .map(|s| s.to_string())
                });

            // 提取价格
            let price = cell.select(&price_selector)
                .next()
                .map(|e| {
                    let price_text = e.text().collect::<String>().trim().to_string();
                    if price_text.is_empty() {
                        "Free".to_string()
                    } else {
                        format!("${}", price_text)
                    }
                })
                .unwrap_or_else(|| "Free".to_string());

            // 提取描述
            let short_text = cell.select(&text_selector)
                .next()
                .and_then(|e| e.value().attr("title").or_else(|| {
                    let text = e.text().collect::<String>();
                    if text.is_empty() { None } else { Some(Box::leak(text.into_boxed_str()) as &str) }
                }))
                .unwrap_or("")
                .to_string();

            // 从URL生成ID
            let id = url.split('/')
                .last()
                .unwrap_or(&format!("itch_{}", assets.len()))
                .to_string();

            assets.push(ItchAsset {
                id,
                title,
                author,
                url,
                cover_url,
                price,
                short_text,
                platforms: Vec::new(),
                tags: Vec::new(),
                source: "itch.io".to_string(),
            });
        }

        assets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let client = ItchClient::new();
        let result = client.search("pixel art", 1).await;
        
        if let Ok(assets) = result {
            println!("Found {} assets", assets.len());
            for asset in assets.iter().take(3) {
                println!("  - {} by {} ({})", asset.title, asset.author, asset.price);
                if let Some(ref cover) = asset.cover_url {
                    println!("    Cover: {}", cover);
                }
            }
        }
    }
}
