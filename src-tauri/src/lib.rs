use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Manager, State};

struct DbLock(Mutex<()>);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalPlayerPayload {
    _playlist_id: Option<String>,
    _playlist_item_id: Option<String>,
    _playback_rate: Option<f64>,
    _playlist_reverse: Option<bool>,
    _playlist_shuffle: Option<bool>,
    _playlist_loop: Option<bool>,
    _url: Option<String>,
    _video_id: Option<String>,
    _timestamp: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpRequestPayload {
    url: String,
    method: Option<String>,
    headers: Option<Map<String, Value>>,
    body: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HttpResponsePayload {
    status: u16,
    status_text: String,
    headers: Vec<(String, String)>,
    body: String,
}

#[derive(Debug, Serialize)]
struct UnsupportedCommand {
    command: &'static str,
    reason: &'static str,
}

fn unsupported(command: &'static str, reason: &'static str) -> String {
    serde_json::to_string(&UnsupportedCommand { command, reason })
        .unwrap_or_else(|_| format!("{command}: {reason}"))
}

fn format_reqwest_error(error: reqwest::Error) -> String {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(&error);

    while let Some(error) = source {
        message.push_str(": ");
        message.push_str(&error.to_string());
        source = error.source();
    }

    message
}

fn player_cache_path(app: &AppHandle, key: &str) -> Result<PathBuf, String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("player-cache");

    fs::create_dir_all(&cache_dir).map_err(|error| error.to_string())?;

    let safe_key = key
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();

    Ok(cache_dir.join(safe_key))
}

fn store_path(app: &AppHandle, store: &str) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("stores");

    fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;

    Ok(data_dir.join(format!("{store}.json")))
}

fn read_store(app: &AppHandle, store: &str) -> Result<Vec<Value>, String> {
    let path = store_path(app, store)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let bytes = fs::read(path).map_err(|error| error.to_string())?;

    if bytes.is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

fn write_store(app: &AppHandle, store: &str, docs: &[Value]) -> Result<(), String> {
    let path = store_path(app, store)?;
    let json = serde_json::to_vec_pretty(docs).map_err(|error| error.to_string())?;

    fs::write(path, json).map_err(|error| error.to_string())
}

fn as_object_mut(value: &mut Value) -> Result<&mut Map<String, Value>, String> {
    value
        .as_object_mut()
        .ok_or_else(|| "expected object document".to_owned())
}

fn object_field<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
    value.as_object().and_then(|object| object.get(field))
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    object_field(value, field).and_then(Value::as_str)
}

fn data_object(data: Option<Value>) -> Result<Value, String> {
    data.filter(Value::is_object)
        .ok_or_else(|| "database action requires object data".to_owned())
}

fn data_string(data: Option<Value>) -> Result<String, String> {
    data.and_then(|value| value.as_str().map(str::to_owned))
        .ok_or_else(|| "database action requires string data".to_owned())
}

fn upsert_by_field(docs: &mut Vec<Value>, field: &str, field_value: &Value, doc: Value) {
    if let Some(existing) = docs
        .iter_mut()
        .find(|existing| object_field(existing, field) == Some(field_value))
    {
        *existing = doc;
    } else {
        docs.push(doc);
    }
}

fn upsert_set_fields(
    docs: &mut Vec<Value>,
    id: &str,
    fields: Map<String, Value>,
) -> Result<(), String> {
    if let Some(existing) = docs
        .iter_mut()
        .find(|doc| string_field(doc, "_id") == Some(id))
    {
        let existing = as_object_mut(existing)?;

        for (key, value) in fields {
            existing.insert(key, value);
        }
    } else {
        let mut doc = Map::new();
        doc.insert("_id".to_owned(), Value::String(id.to_owned()));

        for (key, value) in fields {
            doc.insert(key, value);
        }

        docs.push(Value::Object(doc));
    }

    Ok(())
}

fn update_array_field<F>(doc: &mut Value, field: &str, update: F) -> Result<(), String>
where
    F: FnOnce(&mut Vec<Value>),
{
    let object = as_object_mut(doc)?;
    let entry = object
        .entry(field)
        .or_insert_with(|| Value::Array(Vec::new()));
    let array = entry
        .as_array_mut()
        .ok_or_else(|| format!("expected array field \"{field}\""))?;

    update(array);
    Ok(())
}

fn find_or_create_by_id<'a>(docs: &'a mut Vec<Value>, id: &str) -> Result<&'a mut Value, String> {
    if let Some(index) = docs
        .iter()
        .position(|doc| string_field(doc, "_id") == Some(id))
    {
        return Ok(&mut docs[index]);
    }

    docs.push(json!({ "_id": id }));
    docs.last_mut()
        .ok_or_else(|| "failed to create document".to_owned())
}

fn remove_by_id(docs: &mut Vec<Value>, id: &str) {
    docs.retain(|doc| string_field(doc, "_id") != Some(id));
}

fn remove_many_by_ids(docs: &mut Vec<Value>, ids: &[Value]) {
    docs.retain(|doc| !object_field(doc, "_id").is_some_and(|id| ids.contains(id)));
}

fn is_protected(doc: &Value) -> bool {
    object_field(doc, "protected").and_then(Value::as_bool) == Some(true)
}

fn sort_desc_by_number(docs: &mut [Value], field: &str) {
    docs.sort_by(|left, right| {
        let left = object_field(left, field)
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let right = object_field(right, field)
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        right
            .partial_cmp(&left)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn normalized_tauri_settings(docs: &[Value]) -> Vec<Value> {
    const ELECTRON_ONLY_SETTINGS: &[&str] = &[
        "useProxy",
        "proxyProtocol",
        "proxyHostname",
        "proxyPort",
        "proxyUsername",
        "proxyPassword",
        "externalPlayer",
        "externalPlayerExecutable",
        "externalPlayerIgnoreWarnings",
        "externalPlayerIgnoreDefaultArgs",
        "externalPlayerCustomArgs",
        "showAddedExternalPlayerCustomArgs",
        "disableSmoothScrolling",
        "hideToTrayOnMinimize",
        "screenshotAskPath",
        "screenshotFolderPath",
        "bounds",
    ];

    let mut settings = docs
        .iter()
        .filter(|doc| {
            string_field(doc, "_id")
                .map(|id| !ELECTRON_ONLY_SETTINGS.contains(&id))
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();

    upsert_by_field(
        &mut settings,
        "_id",
        &Value::String("backendPreference".to_owned()),
        json!({ "_id": "backendPreference", "value": "invidious" }),
    );
    upsert_by_field(
        &mut settings,
        "_id",
        &Value::String("backendFallback".to_owned()),
        json!({ "_id": "backendFallback", "value": false }),
    );
    upsert_by_field(
        &mut settings,
        "_id",
        &Value::String("proxyVideos".to_owned()),
        json!({ "_id": "proxyVideos", "value": true }),
    );

    settings
}

fn handle_settings(
    action: u32,
    data: Option<Value>,
    docs: &mut Vec<Value>,
) -> Result<Value, String> {
    match action {
        1 => Ok(Value::Array(normalized_tauri_settings(docs))),
        2 => {
            let data = data_object(data)?;

            if string_field(&data, "_id") == Some("screenshotFolderPath") {
                return Ok(Value::Null);
            }

            let id = string_field(&data, "_id")
                .ok_or_else(|| "settings upsert requires _id".to_owned())?
                .to_owned();
            let value = object_field(&data, "value").cloned().unwrap_or(Value::Null);

            upsert_by_field(
                docs,
                "_id",
                &Value::String(id.clone()),
                json!({ "_id": id, "value": value }),
            );
            Ok(Value::Null)
        }
        _ => Err("invalid settings db action".to_owned()),
    }
}

fn handle_history(
    action: u32,
    data: Option<Value>,
    docs: &mut Vec<Value>,
) -> Result<Value, String> {
    match action {
        1 => {
            let mut sorted = docs.clone();
            sort_desc_by_number(&mut sorted, "timeWatched");
            Ok(Value::Array(sorted))
        }
        2 => {
            let data = data_object(data)?;
            let video_id = object_field(&data, "videoId")
                .cloned()
                .ok_or_else(|| "history upsert requires videoId".to_owned())?;

            upsert_by_field(docs, "videoId", &video_id, data);
            Ok(Value::Null)
        }
        3 => {
            let video_id = data_string(data)?;
            docs.retain(|doc| string_field(doc, "videoId") != Some(&video_id));
            Ok(Value::Null)
        }
        5 => {
            docs.clear();
            Ok(Value::Null)
        }
        6 => {
            *docs = data
                .and_then(|value| value.as_array().cloned())
                .ok_or_else(|| "history overwrite requires array data".to_owned())?;
            Ok(Value::Null)
        }
        20 => {
            let data = data_object(data)?;
            let video_id = string_field(&data, "videoId")
                .ok_or_else(|| "watch progress update requires videoId".to_owned())?;
            let watch_progress = object_field(&data, "watchProgress")
                .cloned()
                .unwrap_or(Value::Null);

            if let Some(existing) = docs
                .iter_mut()
                .find(|doc| string_field(doc, "videoId") == Some(video_id))
            {
                as_object_mut(existing)?.insert("watchProgress".to_owned(), watch_progress);
            } else {
                docs.push(json!({ "videoId": video_id, "watchProgress": watch_progress }));
            }

            Ok(Value::Null)
        }
        21 => {
            let data = data_object(data)?;
            let video_id = string_field(&data, "videoId")
                .ok_or_else(|| "playlist update requires videoId".to_owned())?;

            if let Some(existing) = docs
                .iter_mut()
                .find(|doc| string_field(doc, "videoId") == Some(video_id))
            {
                let existing = as_object_mut(existing)?;

                for key in [
                    "lastViewedPlaylistId",
                    "lastViewedPlaylistType",
                    "lastViewedPlaylistItemId",
                ] {
                    existing.insert(
                        key.to_owned(),
                        object_field(&data, key).cloned().unwrap_or(Value::Null),
                    );
                }
            } else {
                docs.push(data);
            }

            Ok(Value::Null)
        }
        _ => Err("invalid history db action".to_owned()),
    }
}

fn handle_profiles(
    action: u32,
    data: Option<Value>,
    docs: &mut Vec<Value>,
) -> Result<Value, String> {
    match action {
        0 => {
            let data = data_object(data)?;
            docs.push(data.clone());
            Ok(data)
        }
        1 => Ok(Value::Array(docs.clone())),
        2 => {
            let data = data_object(data)?;
            let id = object_field(&data, "_id")
                .cloned()
                .ok_or_else(|| "profile upsert requires _id".to_owned())?;

            upsert_by_field(docs, "_id", &id, data);
            Ok(Value::Null)
        }
        3 => {
            let id = data_string(data)?;
            remove_by_id(docs, &id);
            Ok(Value::Null)
        }
        20 => {
            let data = data_object(data)?;
            let channel = object_field(&data, "channel")
                .cloned()
                .ok_or_else(|| "add channel requires channel".to_owned())?;
            let profile_ids = object_field(&data, "profileIds")
                .and_then(Value::as_array)
                .ok_or_else(|| "add channel requires profileIds".to_owned())?;

            for profile_id in profile_ids {
                let Some(profile_id) = profile_id.as_str() else {
                    continue;
                };

                if let Some(profile) = docs
                    .iter_mut()
                    .find(|doc| string_field(doc, "_id") == Some(profile_id))
                {
                    update_array_field(profile, "subscriptions", |subscriptions| {
                        subscriptions.push(channel.clone());
                    })?;
                }
            }

            Ok(Value::Null)
        }
        21 => {
            let data = data_object(data)?;
            let channel_id = string_field(&data, "channelId")
                .ok_or_else(|| "remove channel requires channelId".to_owned())?;
            let profile_ids = object_field(&data, "profileIds")
                .and_then(Value::as_array)
                .ok_or_else(|| "remove channel requires profileIds".to_owned())?;

            for profile_id in profile_ids {
                let Some(profile_id) = profile_id.as_str() else {
                    continue;
                };

                if let Some(profile) = docs
                    .iter_mut()
                    .find(|doc| string_field(doc, "_id") == Some(profile_id))
                {
                    update_array_field(profile, "subscriptions", |subscriptions| {
                        subscriptions
                            .retain(|channel| string_field(channel, "id") != Some(channel_id));
                    })?;
                }
            }

            Ok(Value::Null)
        }
        _ => Err("invalid profile db action".to_owned()),
    }
}

fn handle_playlists(
    action: u32,
    data: Option<Value>,
    docs: &mut Vec<Value>,
) -> Result<Value, String> {
    match action {
        0 => {
            let data = data.ok_or_else(|| "playlist create requires data".to_owned())?;

            if let Some(playlists) = data.as_array() {
                docs.extend(playlists.iter().cloned());
            } else {
                docs.push(data);
            }

            Ok(Value::Null)
        }
        1 => Ok(Value::Array(docs.clone())),
        2 => {
            let data = data_object(data)?;
            let id = object_field(&data, "_id")
                .cloned()
                .ok_or_else(|| "playlist upsert requires _id".to_owned())?;

            upsert_by_field(docs, "_id", &id, data);
            Ok(Value::Null)
        }
        3 => {
            let id = data_string(data)?;
            docs.retain(|doc| string_field(doc, "_id") != Some(&id) || is_protected(doc));
            Ok(Value::Null)
        }
        4 => {
            let ids = data
                .and_then(|value| value.as_array().cloned())
                .ok_or_else(|| "playlist delete multiple requires array data".to_owned())?;

            docs.retain(|doc| {
                is_protected(doc) || !object_field(doc, "_id").is_some_and(|id| ids.contains(id))
            });
            Ok(Value::Null)
        }
        5 => {
            docs.clear();
            Ok(Value::Null)
        }
        20 | 21 => {
            let data = data_object(data)?;
            let id = string_field(&data, "_id")
                .ok_or_else(|| "playlist video upsert requires _id".to_owned())?;
            let last_updated_at = object_field(&data, "lastUpdatedAt")
                .cloned()
                .unwrap_or(Value::Null);
            let playlist = find_or_create_by_id(docs, id)?;

            update_array_field(playlist, "videos", |videos| {
                if action == 20 {
                    if let Some(video_data) = object_field(&data, "videoData") {
                        videos.push(video_data.clone());
                    }
                } else if let Some(new_videos) =
                    object_field(&data, "videos").and_then(Value::as_array)
                {
                    videos.extend(new_videos.iter().cloned());
                }
            })?;

            as_object_mut(playlist)?.insert("lastUpdatedAt".to_owned(), last_updated_at);
            Ok(Value::Null)
        }
        22 => {
            let data = data_object(data)?;
            let id = string_field(&data, "_id")
                .ok_or_else(|| "playlist video delete requires _id".to_owned())?;
            let last_updated_at = object_field(&data, "lastUpdatedAt")
                .cloned()
                .unwrap_or(Value::Null);
            let playlist = find_or_create_by_id(docs, id)?;
            let playlist_item_id = string_field(&data, "playlistItemId").map(str::to_owned);
            let video_id = string_field(&data, "videoId").map(str::to_owned);

            update_array_field(playlist, "videos", |videos| {
                videos.retain(|video| {
                    if let Some(playlist_item_id) = playlist_item_id.as_deref() {
                        return string_field(video, "playlistItemId") != Some(playlist_item_id);
                    }

                    if let Some(video_id) = video_id.as_deref() {
                        return string_field(video, "videoId") != Some(video_id);
                    }

                    true
                });
            })?;

            as_object_mut(playlist)?.insert("lastUpdatedAt".to_owned(), last_updated_at);
            Ok(Value::Null)
        }
        23 => {
            let data = data_object(data)?;
            let id = string_field(&data, "_id")
                .ok_or_else(|| "playlist video delete requires _id".to_owned())?;
            let last_updated_at = object_field(&data, "lastUpdatedAt")
                .cloned()
                .unwrap_or(Value::Null);
            let playlist_item_ids = object_field(&data, "playlistItemIds")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let playlist = find_or_create_by_id(docs, id)?;

            update_array_field(playlist, "videos", |videos| {
                videos.retain(|video| {
                    !object_field(video, "playlistItemId")
                        .is_some_and(|id| playlist_item_ids.contains(id))
                });
            })?;

            as_object_mut(playlist)?.insert("lastUpdatedAt".to_owned(), last_updated_at);
            Ok(Value::Null)
        }
        24 => {
            let id = data_string(data)?;
            let playlist = find_or_create_by_id(docs, &id)?;
            as_object_mut(playlist)?.insert("videos".to_owned(), Value::Array(Vec::new()));
            Ok(Value::Null)
        }
        _ => Err("invalid playlist db action".to_owned()),
    }
}

fn handle_search_history(
    action: u32,
    data: Option<Value>,
    docs: &mut Vec<Value>,
) -> Result<Value, String> {
    match action {
        1 => {
            let mut sorted = docs.clone();
            sort_desc_by_number(&mut sorted, "lastUpdatedAt");
            Ok(Value::Array(sorted))
        }
        2 => {
            let data = data_object(data)?;
            let id = object_field(&data, "_id")
                .cloned()
                .ok_or_else(|| "search history upsert requires _id".to_owned())?;

            upsert_by_field(docs, "_id", &id, data);
            Ok(Value::Null)
        }
        3 => {
            let id = data_string(data)?;
            remove_by_id(docs, &id);
            Ok(Value::Null)
        }
        5 => {
            docs.clear();
            Ok(Value::Null)
        }
        6 => {
            *docs = data
                .and_then(|value| value.as_array().cloned())
                .ok_or_else(|| "search history overwrite requires array data".to_owned())?;
            Ok(Value::Null)
        }
        _ => Err("invalid search history db action".to_owned()),
    }
}

fn handle_subscription_cache(
    action: u32,
    data: Option<Value>,
    docs: &mut Vec<Value>,
) -> Result<Value, String> {
    match action {
        1 => Ok(Value::Array(docs.clone())),
        4 => {
            let ids = data
                .and_then(|value| value.as_array().cloned())
                .ok_or_else(|| {
                    "subscription cache delete multiple requires array data".to_owned()
                })?;

            remove_many_by_ids(docs, &ids);
            Ok(Value::Null)
        }
        5 => {
            docs.clear();
            Ok(Value::Null)
        }
        20..=24 => {
            let data = data_object(data)?;
            let channel_id = string_field(&data, "channelId")
                .ok_or_else(|| "subscription cache update requires channelId".to_owned())?;
            let entries = object_field(&data, "entries")
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new()));
            let timestamp = object_field(&data, "timestamp")
                .cloned()
                .unwrap_or(Value::Null);

            if action == 23 {
                let Some(doc) = docs
                    .iter_mut()
                    .find(|doc| string_field(doc, "_id") == Some(channel_id))
                else {
                    return Ok(Value::Null);
                };

                let Some(shorts) = object_field(doc, "shorts")
                    .and_then(Value::as_array)
                    .cloned()
                else {
                    return Ok(Value::Null);
                };
                let Some(entries) = entries.as_array() else {
                    return Ok(Value::Null);
                };

                let updated_shorts = shorts
                    .into_iter()
                    .map(|mut cached_video| {
                        let Some(cached_video_id) = string_field(&cached_video, "videoId") else {
                            return cached_video;
                        };

                        let Some(channel_video) = entries
                            .iter()
                            .find(|entry| string_field(entry, "videoId") == Some(cached_video_id))
                        else {
                            return cached_video;
                        };

                        if let Ok(cached_video) = as_object_mut(&mut cached_video) {
                            for key in ["title", "author"] {
                                if let Some(value) = object_field(channel_video, key) {
                                    cached_video.insert(key.to_owned(), value.clone());
                                }
                            }

                            let channel_view_count = object_field(channel_video, "viewCount")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0);
                            let cached_view_count = cached_video
                                .get("viewCount")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0);

                            if channel_view_count > cached_view_count {
                                cached_video.insert(
                                    "viewCount".to_owned(),
                                    object_field(channel_video, "viewCount")
                                        .cloned()
                                        .unwrap_or(Value::Null),
                                );
                            }
                        }

                        cached_video
                    })
                    .collect();

                as_object_mut(doc)?.insert("shorts".to_owned(), Value::Array(updated_shorts));
                return Ok(Value::Null);
            }

            let mut fields = Map::new();
            match action {
                20 => {
                    fields.insert("videos".to_owned(), entries);
                    fields.insert("videosTimestamp".to_owned(), timestamp);
                }
                21 => {
                    fields.insert("liveStreams".to_owned(), entries);
                    fields.insert("liveStreamsTimestamp".to_owned(), timestamp);
                }
                22 => {
                    fields.insert("shorts".to_owned(), entries);
                    fields.insert("shortsTimestamp".to_owned(), timestamp);
                }
                24 => {
                    fields.insert("communityPosts".to_owned(), entries);
                    fields.insert("communityPostsTimestamp".to_owned(), timestamp);
                }
                _ => unreachable!(),
            }

            upsert_set_fields(docs, channel_id, fields)?;
            Ok(Value::Null)
        }
        _ => Err("invalid subscriptionCache db action".to_owned()),
    }
}

#[tauri::command]
fn get_system_locale() -> String {
    std::env::var("LANG")
        .ok()
        .and_then(|locale| locale.split('.').next().map(str::to_owned))
        .filter(|locale| !locale.is_empty())
        .unwrap_or_else(|| "en-US".to_owned())
}

#[tauri::command]
fn enable_proxy(_url: String) -> Result<(), String> {
    Err(unsupported(
        "enable_proxy",
        "proxy configuration is not implemented in the Tauri runtime yet",
    ))
}

#[tauri::command]
fn disable_proxy() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn set_invidious_authorization(
    _authorization: Option<String>,
    _url: Option<String>,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn start_power_save_blocker() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn stop_power_save_blocker() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn player_cache_get(app: AppHandle, key: String) -> Result<Option<Vec<u8>>, String> {
    let path = player_cache_path(&app, &key)?;

    if !path.exists() {
        return Ok(None);
    }

    fs::read(path).map(Some).map_err(|error| error.to_string())
}

#[tauri::command]
fn player_cache_set(app: AppHandle, key: String, value: Vec<u8>) -> Result<(), String> {
    let path = player_cache_path(&app, &key)?;
    fs::write(path, value).map_err(|error| error.to_string())
}

#[tauri::command]
fn generate_po_token(_video_id: String, _context: String) -> Result<String, String> {
    Err(unsupported(
        "generate_po_token",
        "Local API poToken generation still depends on the Electron implementation",
    ))
}

#[tauri::command]
fn relaunch_app(app: AppHandle) {
    app.restart()
}

#[tauri::command]
fn open_in_external_player(_payload: ExternalPlayerPayload) -> Result<(), String> {
    Err(unsupported(
        "open_in_external_player",
        "external player integration must be rebuilt with Android intents",
    ))
}

#[tauri::command]
async fn http_request(payload: HttpRequestPayload) -> Result<HttpResponsePayload, String> {
    let parsed_url = reqwest::Url::parse(&payload.url).map_err(|error| error.to_string())?;

    if parsed_url.scheme() != "https" {
        return Err("only https URLs are allowed".to_owned());
    }

    let method = payload
        .method
        .as_deref()
        .unwrap_or("GET")
        .parse::<reqwest::Method>()
        .map_err(|error| error.to_string())?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("ZKYT Tauri Android")
        .build()
        .map_err(|error| error.to_string())?;

    let mut request = client.request(method, parsed_url);

    if let Some(headers) = payload.headers {
        for (name, value) in headers {
            let Some(value) = value.as_str() else {
                continue;
            };

            let lower_name = name.to_ascii_lowercase();
            if matches!(
                lower_name.as_str(),
                "host" | "connection" | "content-length" | "cookie"
            ) {
                continue;
            }

            let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|error| error.to_string())?;
            let header_value =
                reqwest::header::HeaderValue::from_str(value).map_err(|error| error.to_string())?;
            request = request.header(header_name, header_value);
        }
    }

    if let Some(body) = payload.body {
        request = request.body(body);
    }

    let response = request.send().await.map_err(format_reqwest_error)?;
    let status = response.status();
    let status_text = status.canonical_reason().unwrap_or("").to_owned();
    let headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_owned()))
        })
        .collect::<Vec<_>>();
    let body = response.text().await.map_err(|error| error.to_string())?;

    Ok(HttpResponsePayload {
        status: status.as_u16(),
        status_text,
        headers,
        body,
    })
}

#[tauri::command]
async fn http_get_json(url: String, authorization: Option<String>) -> Result<Value, String> {
    let parsed_url = reqwest::Url::parse(&url).map_err(|error| error.to_string())?;

    if parsed_url.scheme() != "https" {
        return Err("only https URLs are allowed".to_owned());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("ZKYT Tauri Android")
        .build()
        .map_err(|error| error.to_string())?;

    let mut request = client
        .get(parsed_url)
        .header(reqwest::header::ACCEPT, "application/json");

    if let Some(authorization) = authorization {
        request = request.header(reqwest::header::AUTHORIZATION, authorization);
    }

    let response = request.send().await.map_err(format_reqwest_error)?;
    let status = response.status();
    let text = response.text().await.map_err(|error| error.to_string())?;

    if !status.is_success() {
        let body = text.chars().take(800).collect::<String>();
        return Err(format!("HTTP {status}: {body}"));
    }

    serde_json::from_str(&text).map_err(|error| error.to_string())
}

#[tauri::command]
fn db_request(
    app: AppHandle,
    db_lock: State<DbLock>,
    store: String,
    action: u32,
    data: Option<Value>,
) -> Result<Value, String> {
    let _guard = db_lock
        .0
        .lock()
        .map_err(|_| "database lock was poisoned".to_owned())?;
    let mut docs = read_store(&app, &store)?;

    let result = match store.as_str() {
        "settings" => handle_settings(action, data, &mut docs),
        "history" => handle_history(action, data, &mut docs),
        "profiles" => handle_profiles(action, data, &mut docs),
        "playlists" => handle_playlists(action, data, &mut docs),
        "search-history" => handle_search_history(action, data, &mut docs),
        "subscription-cache" => handle_subscription_cache(action, data, &mut docs),
        _ => Err(format!("invalid database store \"{store}\"")),
    }?;

    if action != 1 {
        write_store(&app, &store, &docs)?;
    }

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DbLock(Mutex::new(())))
        .invoke_handler(tauri::generate_handler![
            get_system_locale,
            enable_proxy,
            disable_proxy,
            set_invidious_authorization,
            start_power_save_blocker,
            stop_power_save_blocker,
            player_cache_get,
            player_cache_set,
            generate_po_token,
            relaunch_app,
            open_in_external_player,
            http_request,
            http_get_json,
            db_request,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FreeTube Tauri application");
}
