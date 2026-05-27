use reqwest::Client;
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
            .user_agent("GameAssetStudio/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            download_dir,
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_download(&self, asset_id: &str, asset_title: &str, url: &str) -> DownloadTask {
        let id = format!("dl_{}_{}", asset_id, chrono::Utc::now().timestamp());
        
        // Create safe filename
        let safe_title = asset_title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();
        
        let extension = url.split('.').last().unwrap_or("zip");
        let filename = format!("{}.{}", safe_title, extension);
        let local_path = self.download_dir.join(&filename);

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

    pub async fn start_download(&self, task_id: &str) -> Result<(), String> {
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

        // Spawn download task
        tokio::spawn(async move {
            match Self::download_file(client, &url, &local_path, tasks_clone.clone(), &task_id_clone).await {
                Ok(_) => {
                    let mut tasks = tasks_clone.lock().await;
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id_clone) {
                        task.status = DownloadStatus::Completed;
                        task.progress = 1.0;
                    }
                }
                Err(e) => {
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

    async fn download_file(
        client: Client,
        url: &str,
        local_path: &str,
        tasks: Arc<Mutex<Vec<DownloadTask>>>,
        task_id: &str,
    ) -> Result<(), String> {
        let response = client.get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let total_bytes = response.content_length().unwrap_or(0);
        
        // Update total bytes
        {
            let mut tasks = tasks.lock().await;
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                task.total_bytes = total_bytes;
            }
        }

        let path = Path::new(local_path);
        let mut file = File::create(path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;

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

        Ok(())
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
