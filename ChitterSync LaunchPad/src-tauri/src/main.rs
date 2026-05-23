#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fs;
use std::path::PathBuf;
use tauri::command;
use base64::{engine::general_purpose, Engine as _};
use reqwest::blocking;
use serde_json::Value;

#[derive(serde::Serialize)]
struct Game {
    name: String,
    path: String,
    platform: String,
    icon_base64: Option<String>,
}

/// Fetch image from URL and convert to base64
fn fetch_image_base64(url: &str) -> Option<String> {
    if let Ok(resp) = blocking::get(url) {
        if let Ok(bytes) = resp.bytes() {
            return Some(general_purpose::STANDARD.encode(&bytes));
        }
    }
    None
}

/// Parse Steam appmanifest_xxxx.acf file
fn parse_steam_acf(acf_path: &PathBuf) -> Option<(String, String)> {
    let content = fs::read_to_string(acf_path).ok()?;
    let mut appid = None;
    let mut name = None;
    for line in content.lines() {
        if line.contains("\"appid\"") {
            appid = line.split('"').nth(3).map(|s| s.to_string());
        } else if line.contains("\"name\"") {
            name = line.split('"').nth(3).map(|s| s.to_string());
        }
    }
    Some((appid?, name?))
}

/// Read Steam installed games
fn get_steam_games() -> Vec<Game> {
    let steam_path = r"C:\Program Files (x86)\Steam\steamapps";
    let mut games = Vec::new();

    if let Ok(entries) = fs::read_dir(steam_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(fname) = path.file_name() {
                if fname.to_string_lossy().starts_with("appmanifest_") {
                    if let Some((appid, name)) = parse_steam_acf(&path) {
                        let url = format!(
                            "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/header.jpg",
                            appid
                        );
                        games.push(Game {
                            name,
                            path: path.to_string_lossy().to_string(),
                            platform: "Steam".into(),
                            icon_base64: fetch_image_base64(&url),
                        });
                    }
                }
            }
        }
    }
    games
}

/// Read Epic installed games
fn get_epic_games() -> Vec<Game> {
    let epic_path = r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests";
    let mut games = Vec::new();

    if let Ok(entries) = fs::read_dir(epic_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "item").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                        let name = json
                            .get("AppName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let url = json
                            .get("DisplayImage")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let icon = if !url.is_empty() {
                            fetch_image_base64(url)
                        } else {
                            None
                        };
                        games.push(Game {
                            name,
                            path: path.to_string_lossy().to_string(),
                            platform: "Epic".into(),
                            icon_base64: icon,
                        });
                    }
                }
            }
        }
    }
    games
}

/// Get all installed games
#[command]
fn get_installed_games() -> Vec<Game> {
    let mut games = Vec::new();
    games.extend(get_steam_games());
    games.extend(get_epic_games());

    // Itch placeholder
    games.push(Game {
        name: "Itch.io Game Placeholder".into(),
        path: "".into(),
        platform: "Itch.io".into(),
        icon_base64: None,
    });

    games
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_installed_games])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
