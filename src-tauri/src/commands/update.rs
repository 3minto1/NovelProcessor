use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub(crate) struct UpdateInfo {
    pub has_update: bool,
    pub latest_version: String,
    pub current_version: String,
    pub download_url: String,
    pub release_notes: String,
}

#[tauri::command]
pub(crate) async fn check_update() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/3minto1/NovelProcessor/releases/latest")
        .header("User-Agent", "NovelProcessor")
        .send()
        .await
        .map_err(|e| format!("无法连接 GitHub：{}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API 返回 {}", resp.status()));
    }

    let release: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let tag_name = release["tag_name"].as_str().unwrap_or("").to_string();
    let latest_version = tag_name.trim_start_matches('v').to_string();
    let body = release["body"].as_str().unwrap_or("").to_string();

    let mut download_url = String::new();
    if let Some(assets) = release["assets"].as_array() {
        for asset in assets {
            if let Some(name) = asset["name"].as_str() {
                if name.ends_with(".zip") {
                    download_url = asset["browser_download_url"].as_str().unwrap_or("").to_string();
                    break;
                }
            }
        }
    }

    let has_update = compare_versions(&latest_version, &current_version);

    Ok(UpdateInfo {
        has_update,
        latest_version,
        current_version,
        download_url,
        release_notes: body,
    })
}

#[tauri::command]
pub(crate) async fn download_update(
    state: State<'_, AppState>,
    download_url: String,
    output_dir: String,
) -> Result<String, String> {
    if download_url.is_empty() {
        return Err("下载链接为空".to_string());
    }

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(&download_url)
        .header("User-Agent", "NovelProcessor")
        .send()
        .await
        .map_err(|e| format!("下载失败：{}", e))?;

    if !resp.status().is_success() {
        return Err(format!("下载失败，HTTP {}", resp.status()));
    }

    let file_name = download_url
        .rsplit('/')
        .next()
        .unwrap_or("update.zip")
        .to_string();

    let output_path = std::path::Path::new(&output_dir).join(&file_name);
    let mut file = std::fs::File::create(&output_path).map_err(|e| format!("创建文件失败：{}", e))?;

    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    use std::io::Write;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断：{}", e))?;
        file.write_all(&chunk).map_err(|e| format!("写入失败：{}", e))?;
    }

    Ok(output_path.to_string_lossy().to_string())
}

fn compare_versions(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let latest_parts = parse(latest);
    let current_parts = parse(current);

    for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    latest_parts.len() > current_parts.len()
}
