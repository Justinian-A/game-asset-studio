use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KenneyAsset {
    pub id: String,
    pub title: String,
    pub url: String,
    pub preview_url: Option<String>,
    pub download_url: Option<String>,
    pub license: String,
    pub tags: Vec<String>,
    pub asset_type: String,
    pub source: String,
}

pub struct KenneyClient {
    client: Client,
}

impl KenneyClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    pub async fn search(&self, query: &str, page: u32) -> Result<Vec<KenneyAsset>, Box<dyn Error + Send + Sync>> {
        // Kenney 没有真正的搜索功能，使用标签筛选
        // 将查询词转换为可能的标签
        let tag = self.query_to_tag(query);
        
        let url = if tag.is_empty() {
            // 如果没有匹配的标签，返回首页素材
            format!("https://kenney.nl/assets?page={}", page)
        } else {
            format!("https://kenney.nl/assets/tag:{}?page={}", tag, page)
        };

        log::info!("Searching Kenney: {}", url);

        let response = self.client.get(&url).send().await?;
        let html = response.text().await?;
        let assets = self.parse_search_results(&html);
        
        log::info!("Found {} assets on Kenney", assets.len());
        Ok(assets)
    }

    fn query_to_tag(&self, query: &str) -> String {
        // 将常见搜索词映射到Kenney标签
        let query_lower = query.to_lowercase();
        
        if query_lower.contains("pixel") {
            "pixel".to_string()
        } else if query_lower.contains("platformer") {
            "platformer".to_string()
        } else if query_lower.contains("space") {
            "space".to_string()
        } else if query_lower.contains("dungeon") {
            "dungeon".to_string()
        } else if query_lower.contains("pirate") {
            "pirate".to_string()
        } else if query_lower.contains("ui") || query_lower.contains("interface") {
            "interface".to_string()
        } else if query_lower.contains("button") {
            "button".to_string()
        } else if query_lower.contains("icon") {
            "icon".to_string()
        } else if query_lower.contains("background") {
            "background".to_string()
        } else if query_lower.contains("tile") {
            "tile".to_string()
        } else if query_lower.contains("character") || query_lower.contains("player") {
            "character".to_string()
        } else if query_lower.contains("enemy") || query_lower.contains("monster") {
            "enemy".to_string()
        } else if query_lower.contains("vehicle") || query_lower.contains("car") {
            "vehicle".to_string()
        } else if query_lower.contains("building") || query_lower.contains("house") {
            "building".to_string()
        } else if query_lower.contains("tree") || query_lower.contains("nature") {
            "nature".to_string()
        } else {
            // 没有匹配的标签，返回空
            String::new()
        }
    }

    fn parse_search_results(&self, html: &str) -> Vec<KenneyAsset> {
        let document = Html::parse_document(html);
        let mut assets = Vec::new();

        // Kenney 使用 h3 > a 作为标题链接
        let title_selector = Selector::parse("h3 a, h2 a").unwrap();
        let image_selector = Selector::parse("img").unwrap();

        for title_elem in document.select(&title_selector) {
            let title = title_elem.text().collect::<String>().trim().to_string();
            
            // 过滤掉导航链接
            if title.is_empty() || title.len() < 2 {
                continue;
            }
            
            // 过滤掉明显的非素材链接
            let title_lower = title.to_lowercase();
            if title_lower == "home" || title_lower == "about" || title_lower == "contact" 
                || title_lower == "faq" || title_lower == "license" || title_lower == "donate" {
                continue;
            }

            let url = title_elem.value().attr("href")
                .map(|href| {
                    if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://kenney.nl{}", href)
                    }
                })
                .unwrap_or_default();

            // 只处理/assets/路径的链接
            if !url.contains("/assets/") {
                continue;
            }

            // 从URL生成ID
            let id = url.split('/')
                .last()
                .unwrap_or(&format!("kenney_{}", assets.len()))
                .to_string();

            // 检查是否已存在
            if assets.iter().any(|a: &KenneyAsset| a.id == id) {
                continue;
            }

            assets.push(KenneyAsset {
                id,
                title,
                url,
                preview_url: None, // Kenney的预览图需要额外请求
                download_url: None,
                license: "CC0".to_string(),
                tags: Vec::new(),
                asset_type: "2d".to_string(),
                source: "kenney".to_string(),
            });
        }

        assets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_to_tag() {
        let client = KenneyClient::new();
        
        assert_eq!(client.query_to_tag("pixel art"), "pixel");
        assert_eq!(client.query_to_tag("platformer"), "platformer");
        assert_eq!(client.query_to_tag("ui pack"), "interface");
        assert_eq!(client.query_to_tag("xyzabc"), "");
    }
}
