use tauri::Manager;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

mod db;
mod search;
mod download;
mod image;

#[derive(Serialize, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub sources: Option<Vec<String>>,
    pub page: Option<u32>,
}

#[tauri::command]
async fn search_assets(params: SearchParams) -> Result<Vec<search::Asset>, String> {
    let engine = search::SearchEngine::new();
    let sources = params.sources.unwrap_or_default();
    
    log::info!("Searching for: {} in sources: {:?}", params.query, sources);
    
    let assets = engine.search_all(&params.query, sources).await;
    
    log::info!("Found {} total assets", assets.len());
    Ok(assets)
}

#[tauri::command]
async fn search_source(query: String, source: String, page: u32) -> Result<Vec<search::Asset>, String> {
    let engine = search::SearchEngine::new();
    engine.search_source(&query, &source, page).await
}

#[tauri::command]
async fn start_download(
    app: tauri::AppHandle,
    asset: search::Asset,
) -> Result<download::DownloadTask, String> {
    let download_dir = get_download_dir(&app)?;
    
    // Create download directory if it doesn't exist
    tokio::fs::create_dir_all(&download_dir)
        .await
        .map_err(|e| format!("Failed to create download directory: {}", e))?;
    
    // First, save to library if not already saved
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    
    let record = db::AssetRecord {
        id: 0,
        source_id: asset.id.clone(),
        title: asset.title.clone(),
        author: asset.author.clone(),
        url: asset.url.clone(),
        preview_url: asset.preview_url.clone(),
        local_path: None,
        price: asset.price.clone(),
        license: asset.license.clone(),
        short_text: asset.short_text.clone(),
        tags: asset.tags.join(","),
        asset_type: asset.asset_type.clone(),
        source: asset.source.clone(),
        downloaded: false,
        favorite: false,
        created_at: String::new(),
    };
    
    let asset_db_id = database.insert_asset(&record).map_err(|e| e.to_string())?;
    
    // 获取 API Key（如果有的话）
    let api_key = database.get_setting("itch_api_key").ok().flatten();
    
    // Start download
    let manager = download::DownloadManager::new(download_dir);
    let task = manager.add_download(&asset.id, &asset.title, &asset.url, &asset.source).await;
    let task_id = task.id.clone();
    let tasks_clone = manager.tasks.clone();
    let db_path_clone = db_path.clone();
    let asset_id_clone = asset.id.clone();
    
    manager.start_download(&task_id, api_key).await?;
    
    // Spawn a task to monitor download completion and update database
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            let tasks = tasks_clone.lock().await;
            if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
                match task.status {
                    download::DownloadStatus::Completed => {
                        // Update database
                        if let Ok(database) = db::Database::new(&db_path_clone) {
                            let _ = database.update_downloaded(asset_db_id, true, Some(&task.local_path));
                            log::info!("Asset {} marked as downloaded", asset_id_clone);
                        }
                        break;
                    }
                    download::DownloadStatus::Failed | download::DownloadStatus::Cancelled => {
                        break;
                    }
                    _ => continue,
                }
            } else {
                break;
            }
        }
    });
    
    Ok(task)
}

#[tauri::command]
async fn get_downloads(app: tauri::AppHandle) -> Result<Vec<download::DownloadTask>, String> {
    let download_dir = get_download_dir(&app)?;
    let manager = download::DownloadManager::new(download_dir);
    Ok(manager.get_tasks().await)
}

#[tauri::command]
async fn cancel_download(app: tauri::AppHandle, task_id: String) -> Result<(), String> {
    let download_dir = get_download_dir(&app)?;
    let manager = download::DownloadManager::new(download_dir);
    manager.cancel_download(&task_id).await
}

#[tauri::command]
async fn clear_downloads(app: tauri::AppHandle) -> Result<(), String> {
    let download_dir = get_download_dir(&app)?;
    let manager = download::DownloadManager::new(download_dir);
    manager.clear_completed().await;
    Ok(())
}

// Library commands
#[tauri::command]
async fn save_to_library(app: tauri::AppHandle, asset: search::Asset) -> Result<i64, String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    
    let record = db::AssetRecord {
        id: 0,
        source_id: asset.id.clone(),
        title: asset.title,
        author: asset.author,
        url: asset.url,
        preview_url: asset.preview_url,
        local_path: None,
        price: asset.price,
        license: asset.license,
        short_text: asset.short_text,
        tags: asset.tags.join(","),
        asset_type: asset.asset_type,
        source: asset.source,
        downloaded: false,
        favorite: false,
        created_at: String::new(),
    };
    
    database.insert_asset(&record).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_library_assets(app: tauri::AppHandle, offset: i64, limit: i64) -> Result<Vec<db::AssetRecord>, String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.get_assets(offset, limit).map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_library(app: tauri::AppHandle, query: String) -> Result<Vec<db::AssetRecord>, String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.search_assets(&query).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_favorites(app: tauri::AppHandle) -> Result<Vec<db::AssetRecord>, String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.get_favorites().map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_favorite(app: tauri::AppHandle, id: i64, favorite: bool) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.update_favorite(id, favorite).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_from_library(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.delete_asset(id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_library_stats(app: tauri::AppHandle) -> Result<(i64, i64, i64), String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_downloaded_assets(app: tauri::AppHandle) -> Result<Vec<db::AssetRecord>, String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.get_downloaded().map_err(|e| e.to_string())
}

// itch.io API commands
#[derive(serde::Serialize)]
struct ApiKeyValidationResult {
    valid: bool,
    username: Option<String>,
    error: Option<String>,
}

#[tauri::command]
async fn validate_itch_api_key(api_key: String) -> Result<ApiKeyValidationResult, String> {
    let client = reqwest::Client::new();
    
    // 调用 itch.io API 验证 Key
    let response = client
        .get("https://api.itch.io/credentials/info")
        .header("Authorization", &api_key)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;
    
    if response.status().is_success() {
        let body: serde_json::Value = response.json().await
            .map_err(|e| format!("解析响应失败: {}", e))?;
        
        let username = body.get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(ApiKeyValidationResult {
            valid: true,
            username,
            error: None,
        })
    } else {
        Ok(ApiKeyValidationResult {
            valid: false,
            username: None,
            error: Some("API Key 无效".to_string()),
        })
    }
}

#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
async fn set_itch_api_key(app: tauri::AppHandle, api_key: String) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.set_setting("itch_api_key", &api_key).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_itch_api_key(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.get_setting("itch_api_key").map_err(|e| e.to_string())
}

#[tauri::command]
async fn is_setup_complete(app: tauri::AppHandle) -> Result<bool, String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    
    // 检查是否已经配置了 API Key
    match database.get_setting("itch_api_key").map_err(|e| e.to_string())? {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

// Image processing commands
#[tauri::command]
async fn get_image_info(path: String) -> Result<image::ImageInfo, String> {
    let processor = image::ImageProcessor::new();
    processor.get_image_info(std::path::Path::new(&path))
}

#[tauri::command]
async fn convert_image(
    input_path: String,
    output_dir: String,
    target_format: String,
) -> Result<image::ProcessResult, String> {
    let processor = image::ImageProcessor::new();
    let options = image::ProcessOptions {
        target_format: Some(target_format),
        max_width: None,
        max_height: None,
        quality: None,
        remove_background: false,
    };
    processor.process_file(
        std::path::Path::new(&input_path),
        std::path::Path::new(&output_dir),
        &options,
    )
}

#[tauri::command]
async fn resize_image(
    input_path: String,
    output_dir: String,
    max_width: u32,
    max_height: u32,
) -> Result<image::ProcessResult, String> {
    let processor = image::ImageProcessor::new();
    processor.resize_image(
        std::path::Path::new(&input_path),
        std::path::Path::new(&output_dir),
        max_width,
        max_height,
    )
}

#[tauri::command]
async fn split_spritesheet(
    input_path: String,
    output_dir: String,
    frame_width: u32,
    frame_height: u32,
) -> Result<Vec<image::ProcessResult>, String> {
    let processor = image::ImageProcessor::new();
    let options = image::SpriteSheetOptions {
        frame_width,
        frame_height,
        columns: None,
        padding: None,
    };
    processor.split_spritesheet(
        std::path::Path::new(&input_path),
        std::path::Path::new(&output_dir),
        &options,
    )
}

fn get_download_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // 尝试从数据库读取用户配置的下载路径
    let db_path = get_db_path(app)?;
    if let Ok(database) = db::Database::new(&db_path) {
        if let Ok(Some(path)) = database.get_setting("download_dir") {
            let dir = PathBuf::from(&path);
            // 确保目录存在
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("[Download] Failed to create download dir: {}", e);
            }
            return Ok(dir);
        }
    }
    
    // 默认路径
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let download_dir = home_dir.join("GameAssetStudio").join("downloads");
    Ok(download_dir)
}

#[tauri::command]
fn set_download_dir(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let database = db::Database::new(&db_path).map_err(|e| e.to_string())?;
    database.set_setting("download_dir", &path).map_err(|e| e.to_string())?;
    
    // 确保目录存在
    let dir = PathBuf::from(&path);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    
    eprintln!("[Settings] Download directory set to: {}", path);
    Ok(())
}

#[tauri::command]
fn get_download_path(app: tauri::AppHandle) -> Result<String, String> {
    let dir = get_download_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}

fn get_db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(app_dir.join("assets.db"))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Game Asset Studio.", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize database
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            
            let db_path = app_dir.join("assets.db");
            db::init_database(&db_path).expect("failed to initialize database");
            
            // Log setup - always enable logging
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;
            
            log::info!("Game Asset Studio started");
            log::info!("Database path: {:?}", db_path);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            search_assets,
            search_source,
            start_download,
            get_downloads,
            cancel_download,
            clear_downloads,
            save_to_library,
            get_library_assets,
            search_library,
            get_favorites,
            toggle_favorite,
            delete_from_library,
            get_library_stats,
            get_downloaded_assets,
            set_download_dir,
            get_download_path,
            validate_itch_api_key,
            set_itch_api_key,
            get_itch_api_key,
            is_setup_complete,
            open_url,
            get_image_info,
            convert_image,
            resize_image,
            split_spritesheet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
