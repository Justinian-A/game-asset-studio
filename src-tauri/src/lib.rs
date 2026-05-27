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
    
    // Start download
    let manager = download::DownloadManager::new(download_dir);
    let task = manager.add_download(&asset.id, &asset.title, &asset.url).await;
    let task_id = task.id.clone();
    let tasks_clone = manager.tasks.clone();
    let db_path_clone = db_path.clone();
    let asset_id_clone = asset.id.clone();
    
    manager.start_download(&task_id).await?;
    
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

fn get_download_dir(_app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let download_dir = home_dir.join("GameAssetStudio").join("downloads");
    Ok(download_dir)
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
            
            // Log setup
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            
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
            get_image_info,
            convert_image,
            resize_image,
            split_spritesheet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
