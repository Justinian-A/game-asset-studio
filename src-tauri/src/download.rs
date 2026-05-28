use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadTask {
    pub id: String,
    pub asset_id: String,
    pub asset_title: String,
    pub url: String,
    pub local_path: String,
    pub status: DownloadStatus,
    pub progress: f32,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

pub struct DownloadManager {
    client: Client,
    download_dir: PathBuf,
    pub tasks: Arc<Mutex<Vec<DownloadTask>>>,
}

impl DownloadManager {
    pub fn new(download_dir: PathBuf) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            download_dir,
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_download(&self, asset_id: &str, asset_title: &str, url: &str, source: &str) -> DownloadTask {
        let id = format!("dl_{}_{}", asset_id, chrono::Utc::now().timestamp());
        
        // Create safe filename
        let safe_title = asset_title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();
        
        // 默认使用zip扩展名
        let filename = format!("{}.zip", safe_title);
        let local_path = self.download_dir.join(&filename);

        log::info!("[Download] Added download task: {} -> {}", asset_title, url);

        let task = DownloadTask {
            id: id.clone(),
            asset_id: asset_id.to_string(),
            asset_title: asset_title.to_string(),
            url: url.to_string(),
            local_path: local_path.to_string_lossy().to_string(),
            status: DownloadStatus::Pending,
            progress: 0.0,
            total_bytes: 0,
            downloaded_bytes: 0,
            error: None,
        };

        let mut tasks = self.tasks.lock().await;
        tasks.push(task.clone());
        
        task
    }

    pub async fn start_download(&self, task_id: &str, api_key: Option<String>) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;
        let task = tasks.iter_mut().find(|t| t.id == task_id);
        
        let task = match task {
            Some(t) => t,
            None => return Err("Task not found".to_string()),
        };

        task.status = DownloadStatus::Downloading;
        let url = task.url.clone();
        let local_path = task.local_path.clone();
        let tasks_clone = self.tasks.clone();
        let task_id_clone = task_id.to_string();
        let client = self.client.clone();
        let download_dir = self.download_dir.clone();

        eprintln!("[Download] Starting download: {}", url);

        // Spawn download task
        tokio::spawn(async move {
            eprintln!("[Download] Starting download task for: {}", url);
            match Self::download_asset(client, &url, &local_path, &download_dir, tasks_clone.clone(), &task_id_clone, api_key).await {
                Ok(final_path) => {
                    eprintln!("[Download] SUCCESS: File saved to {}", final_path);
                    let mut tasks = tasks_clone.lock().await;
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id_clone) {
                        task.status = DownloadStatus::Completed;
                        task.progress = 1.0;
                        task.local_path = final_path;
                    }
                }
                Err(e) => {
                    eprintln!("[Download] FAILED: {}", e);
                    let mut tasks = tasks_clone.lock().await;
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id_clone) {
                        task.status = DownloadStatus::Failed;
                        task.error = Some(e);
                    }
                }
            }
        });

        Ok(())
    }

    /// 下载素材 - 使用 itch.io API 或从页面提取下载链接
    async fn download_asset(
        client: Client,
        page_url: &str,
        local_path: &str,
        download_dir: &Path,
        tasks: Arc<Mutex<Vec<DownloadTask>>>,
        task_id: &str,
        api_key: Option<String>,
    ) -> Result<String, String> {
        eprintln!("[Download] Starting download: {}", page_url);
        
        // 如果是 itch.io 且有 API Key，使用 API
        if page_url.contains("itch.io") {
            if let Some(ref key) = api_key {
                eprintln!("[Download] Using itch.io API");
                return Self::download_from_itch_api(client, page_url, local_path, download_dir, tasks, task_id, key).await;
            }
        }
        
        // 否则使用网页解析方式
        eprintln!("[Download] Using web scraping");
        Self::download_from_web(client, page_url, local_path, download_dir, tasks, task_id).await
    }

    /// 使用 itch.io API 下载
    async fn download_from_itch_api(
        client: Client,
        page_url: &str,
        local_path: &str,
        download_dir: &Path,
        tasks: Arc<Mutex<Vec<DownloadTask>>>,
        task_id: &str,
        api_key: &str,
    ) -> Result<String, String> {
        // 1. 从 URL 提取游戏 slug
        let slug = page_url.split('/').last().unwrap_or("");
        eprintln!("[Download] Game slug: {}", slug);

        // 2. 搜索游戏 ID
        let search_url = format!("https://api.itch.io/games/{}", slug);
        let response = client.get(&search_url)
            .header("Authorization", api_key)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            // 如果 API 失败，回退到网页解析
            eprintln!("[Download] API failed, falling back to web scraping");
            return Self::download_from_web(client, page_url, local_path, download_dir, tasks, task_id).await;
        }

        let game_data: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        let game_id = game_data.get("game")
            .and_then(|g| g.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or("Game ID not found in API response")?;

        eprintln!("[Download] Game ID: {}", game_id);

        // 3. 获取上传列表
        let uploads_url = format!("https://api.itch.io/games/{}/uploads", game_id);
        let response = client.get(&uploads_url)
            .header("Authorization", api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to get uploads: {}", e))?;

        if !response.status().is_success() {
            return Self::download_from_web(client, page_url, local_path, download_dir, tasks, task_id).await;
        }

        let uploads_data: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse uploads: {}", e))?;

        let upload_id = uploads_data.get("uploads")
            .and_then(|u| u.as_array())
            .and_then(|arr| arr.first())
            .and_then(|upload| upload.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or("No uploads found")?;

        eprintln!("[Download] Upload ID: {}", upload_id);

        // 4. 获取下载链接
        let download_url = format!("https://api.itch.io/uploads/{}/download", upload_id);
        let response = client.get(&download_url)
            .header("Authorization", api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to get download URL: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download API returned: {}", response.status()));
        }

        let download_data: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse download response: {}", e))?;

        let file_url = download_data.get("url")
            .and_then(|u| u.as_str())
            .ok_or("No download URL in response")?
            .to_string();

        eprintln!("[Download] File URL: {}", file_url);

        // 5. 下载文件
        Self::download_file(client, &file_url, local_path, download_dir, tasks, task_id).await
    }

    /// 从网页下载（备用方案）
    async fn download_from_web(
        client: Client,
        page_url: &str,
        local_path: &str,
        download_dir: &Path,
        tasks: Arc<Mutex<Vec<DownloadTask>>>,
        task_id: &str,
    ) -> Result<String, String> {
        eprintln!("[Download] Fetching page: {}", page_url);
        
        let response = client.get(page_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch page: {}", e))?;

        let status = response.status();
        eprintln!("[Download] Page response status: {}", status);

        if !status.is_success() {
            return Err(format!("HTTP error: {}", status));
        }

        let html = response.text().await
            .map_err(|e| format!("Failed to read page: {}", e))?;
        
        eprintln!("[Download] Page HTML length: {} bytes", html.len());

        let download_url = Self::extract_download_url(&html, page_url)?;
        eprintln!("[Download] Extracted download URL: {}", download_url);

        Self::download_file(client, &download_url, local_path, download_dir, tasks, task_id).await
    }

    /// 从HTML中提取下载链接
    fn extract_download_url(html: &str, page_url: &str) -> Result<String, String> {
        let document = Html::parse_document(html);

        eprintln!("[Download] Extracting download URL from page: {}", page_url);
        eprintln!("[Download] HTML length: {} bytes", html.len());

        // 尝试多种方式提取下载链接
        
        // 方式1: 查找 download 按钮链接
        let download_selectors = [
            "a[href*='/download']",
            ".download_btn",
            ".button.download",
            "a.download",
            "[data-action='download']",
        ];
        
        for selector_str in &download_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(elem) = document.select(&selector).next() {
                    if let Some(href) = elem.value().attr("href") {
                        eprintln!("[Download] Found download link with selector '{}': {}", selector_str, href);
                        let url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            let base = page_url.trim_end_matches('/');
                            format!("{}{}", base, href)
                        };
                        return Ok(url);
                    }
                }
            }
        }

        // 方式2: 查找 upload_list 中的下载链接
        if let Ok(upload_selector) = Selector::parse(".upload_list a, .upload a, a[href*='upload']") {
            for elem in document.select(&upload_selector) {
                if let Some(href) = elem.value().attr("href") {
                    eprintln!("[Download] Found upload link: {}", href);
                    if href.contains("/upload/") || href.contains("download") {
                        let url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("https://itch.io{}", href)
                        };
                        return Ok(url);
                    }
                }
            }
        }

        // 方式3: 查找所有包含 download 的链接
        if let Ok(all_links) = Selector::parse("a[href]") {
            for elem in document.select(&all_links) {
                if let Some(href) = elem.value().attr("href") {
                    if href.contains("/download") && !href.contains("javascript") {
                        eprintln!("[Download] Found download link in page: {}", href);
                        let url = if href.starts_with("http") {
                            href.to_string()
                        } else if href.starts_with("/") {
                            format!("https://itch.io{}", href)
                        } else {
                            format!("{}/{}", page_url.trim_end_matches('/'), href)
                        };
                        return Ok(url);
                    }
                }
            }
        }

        // 方式4: 如果是免费素材，尝试直接访问 /download 路径
        if page_url.contains("itch.io") {
            let download_url = format!("{}/download", page_url.trim_end_matches('/'));
            eprintln!("[Download] Trying direct download URL: {}", download_url);
            return Ok(download_url);
        }

        eprintln!("[Download] ERROR: Could not extract download URL from page");
        Err("Could not extract download URL from page".to_string())
    }

    /// 下载文件到本地
    async fn download_file(
        client: Client,
        url: &str,
        local_path: &str,
        download_dir: &Path,
        tasks: Arc<Mutex<Vec<DownloadTask>>>,
        task_id: &str,
    ) -> Result<String, String> {
        log::info!("[Download] Downloading file from: {}", url);
        
        let response = client.get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        log::info!("[Download] File download response status: {}", status);

        if !status.is_success() {
            // 如果是重定向，可能是需要登录或付费
            if status.as_u16() == 302 || status.as_u16() == 301 {
                if let Some(location) = response.headers().get("location") {
                    log::warn!("[Download] Redirect to: {:?}", location);
                }
            }
            return Err(format!("HTTP error: {} - 可能需要登录或付费", status));
        }

        // 从响应头获取文件名和大小
        let total_bytes = response.content_length().unwrap_or(0);
        log::info!("[Download] File size: {} bytes", total_bytes);
        
        // 尝试从 Content-Disposition 获取文件名
        let filename = response.headers()
            .get("content-disposition")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                v.split("filename=")
                    .last()
                    .map(|f| f.trim_matches('"').trim().to_string())
            })
            .unwrap_or_else(|| {
                // 从URL提取文件名
                url.split('/')
                    .last()
                    .unwrap_or("download.zip")
                    .split('?')
                    .next()
                    .unwrap_or("download.zip")
                    .to_string()
            });

        log::info!("[Download] Filename: {}", filename);

        // 构建最终保存路径
        let final_path = download_dir.join(&filename);
        log::info!("[Download] Save path: {:?}", final_path);
        
        // Update total bytes
        {
            let mut tasks = tasks.lock().await;
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                task.total_bytes = total_bytes;
                task.local_path = final_path.to_string_lossy().to_string();
            }
        }

        // 确保目录存在
        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let mut file = File::create(&final_path)
            .await
            .map_err(|e| format!("Failed to create file {}: {}", final_path.display(), e))?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        use futures_util::StreamExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Write error: {}", e))?;
            
            downloaded += chunk.len() as u64;
            let progress = if total_bytes > 0 {
                downloaded as f32 / total_bytes as f32
            } else {
                0.0
            };

            // Update progress
            let mut tasks = tasks.lock().await;
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                task.downloaded_bytes = downloaded;
                task.progress = progress;
            }
        }

        file.flush()
            .await
            .map_err(|e| format!("Flush error: {}", e))?;

        log::info!("[Download] Download complete: {} bytes", downloaded);
        Ok(final_path.to_string_lossy().to_string())
    }

    pub async fn get_tasks(&self) -> Vec<DownloadTask> {
        let tasks = self.tasks.lock().await;
        tasks.clone()
    }

    pub async fn get_task(&self, task_id: &str) -> Option<DownloadTask> {
        let tasks = self.tasks.lock().await;
        tasks.iter().find(|t| t.id == task_id).cloned()
    }

    pub async fn cancel_download(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = DownloadStatus::Cancelled;
            Ok(())
        } else {
            Err("Task not found".to_string())
        }
    }

    pub async fn remove_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;
        let len_before = tasks.len();
        tasks.retain(|t| t.id != task_id);
        
        if tasks.len() < len_before {
            Ok(())
        } else {
            Err("Task not found".to_string())
        }
    }

    pub async fn clear_completed(&self) {
        let mut tasks = self.tasks.lock().await;
        tasks.retain(|t| t.status != DownloadStatus::Completed && t.status != DownloadStatus::Failed);
    }
}
