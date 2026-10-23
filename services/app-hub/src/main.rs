use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{env, fs, net::SocketAddr, path::{Path as FsPath, PathBuf}, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};

const MAX_PACKAGE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
    storage: Arc<PathBuf>,
}

#[derive(Serialize)]
struct AppRecord {
    id: String,
    title: String,
    version: String,
    package_type: String,
    description: String,
    publisher: String,
    category: String,
    filename: String,
    sha256: String,
    size_bytes: i64,
    created_at: String,
}

#[derive(Deserialize)]
struct SearchQuery { q: Option<String>, limit: Option<i64> }

#[derive(Serialize)]
struct ErrorBody { error: String }

fn error(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(ErrorBody { error: message.into() })).into_response()
}

fn safe_component(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max &&
        s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) &&
        !s.contains("..") && !s.starts_with('.')
}

fn init_db(path: &FsPath) -> Result<Connection, rusqlite::Error> {
    let db = Connection::open(path)?;
    db.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS apps (
           id TEXT PRIMARY KEY NOT NULL,
           title TEXT NOT NULL,
           version TEXT NOT NULL,
           package_type TEXT NOT NULL CHECK(package_type IN ('LAP','LUAP')),
           description TEXT NOT NULL,
           publisher TEXT NOT NULL,
           category TEXT NOT NULL,
           filename TEXT NOT NULL,
           sha256 TEXT NOT NULL,
           size_bytes INTEGER NOT NULL,
           created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );
         CREATE INDEX IF NOT EXISTS apps_title_idx ON apps(title);
         CREATE INDEX IF NOT EXISTS apps_category_idx ON apps(category);"
    )?;
    Ok(db)
}

fn read_apps(db: &Connection, query: &str, limit: i64) -> Result<Vec<AppRecord>, rusqlite::Error> {
    let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
    let mut stmt = db.prepare(
        "SELECT id,title,version,package_type,description,publisher,category,filename,sha256,size_bytes,created_at
         FROM apps WHERE (?1 = '' OR title LIKE ?2 ESCAPE '\\' OR description LIKE ?2 ESCAPE '\\' OR publisher LIKE ?2 ESCAPE '\\' OR category LIKE ?2 ESCAPE '\\')
         ORDER BY created_at DESC, title COLLATE NOCASE LIMIT ?3"
    )?;
    let rows = stmt.query_map(params![query, pattern, limit], |r| Ok(AppRecord {
        id:r.get(0)?, title:r.get(1)?, version:r.get(2)?, package_type:r.get(3)?,
        description:r.get(4)?, publisher:r.get(5)?, category:r.get(6)?, filename:r.get(7)?,
        sha256:r.get(8)?, size_bytes:r.get(9)?, created_at:r.get(10)?,
    }))?;
    rows.collect()
}

async fn list_apps(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> Response {
    let query = q.q.unwrap_or_default();
    if query.len() > 120 { return error(StatusCode::BAD_REQUEST, "Search query too long"); }
    let limit = q.limit.unwrap_or(50).clamp(1, 100);
    let db = match state.db.lock() { Ok(v) => v, Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Database lock failed") };
    match read_apps(&db, &query, limit) {
        Ok(apps) => Json(apps).into_response(),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "Database query failed"),
    }
}

async fn upload_app(State(state): State<AppState>, mut multipart: Multipart) -> Response {
    let mut title = String::new();
    let mut version = "1.0.0".to_string();
    let mut description = String::new();
    let mut publisher = String::new();
    let mut category = "Other".to_string();
    let mut package_name = String::new();
    let mut package_bytes: Option<Vec<u8>> = None;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(_) => return error(StatusCode::BAD_REQUEST, "Malformed multipart upload"),
        };
        let name = field.name().unwrap_or("").to_string();
        let filename = field.file_name().unwrap_or("").to_string();
        if name == "package" {
            let ext = FsPath::new(&filename).extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext != "lap" && ext != "luap" {
                return error(StatusCode::BAD_REQUEST, "Package must use .lap or .luap extension");
            }
            if filename.len() > 180 || filename.contains('/') || filename.contains('\\') {
                return error(StatusCode::BAD_REQUEST, "Unsafe package filename");
            }
            let bytes = match field.bytes().await {
                Ok(v) if !v.is_empty() && v.len() <= MAX_PACKAGE_BYTES => v,
                Ok(_) => return error(StatusCode::PAYLOAD_TOO_LARGE, "Package must be between 1 byte and 64 MiB"),
                Err(_) => return error(StatusCode::BAD_REQUEST, "Could not read package upload"),
            };
            if bytes.len() < 4 || &bytes[..2] != b"PK" {
                return error(StatusCode::BAD_REQUEST, "Package does not look like a ZIP archive");
            }
            package_name = filename;
            package_bytes = Some(bytes.to_vec());
        } else {
            let value = match field.text().await {
                Ok(v) if v.len() <= 4000 => v,
                Ok(_) => return error(StatusCode::BAD_REQUEST, "Metadata field too long"),
                Err(_) => return error(StatusCode::BAD_REQUEST, "Invalid metadata field"),
            };
            match name.as_str() {
                "title" => title = value,
                "version" => version = value,
                "description" => description = value,
                "publisher" => publisher = value,
                "category" => category = value,
                _ => {}
            }
        }
    }

    if title.trim().is_empty() || title.len() > 120 { return error(StatusCode::BAD_REQUEST, "title is required (max 120 bytes)"); }
    if publisher.trim().is_empty() || publisher.len() > 100 { return error(StatusCode::BAD_REQUEST, "publisher is required (max 100 bytes)"); }
    if version.trim().is_empty() || version.len() > 40 { return error(StatusCode::BAD_REQUEST, "version is required (max 40 bytes)"); }
    if description.len() > 2000 || category.trim().is_empty() || category.len() > 60 {
        return error(StatusCode::BAD_REQUEST, "Description or category exceeds limits");
    }
    let bytes = match package_bytes { Some(v) => v, None => return error(StatusCode::BAD_REQUEST, "package file is required") };
    let ext = FsPath::new(&package_name).extension().and_then(|s| s.to_str()).unwrap_or("");
    let package_type = if ext == "lap" { "LAP" } else { "LUAP" };
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let slug: String = title.to_ascii_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let slug: String = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    let id = format!("{}-{}", if slug.is_empty() { "app" } else { &slug }, &digest[..12]);
    if !safe_component(&id, 140) { return error(StatusCode::BAD_REQUEST, "Could not create safe app ID"); }
    let stored_name = format!("{}-{}.{}", id, stamp, ext);
    let destination = state.storage.join(&stored_name);
    let temporary = state.storage.join(format!(".upload-{}.tmp", stamp));
    if fs::write(&temporary, &bytes).is_err() {
        return error(StatusCode::INTERNAL_SERVER_ERROR, "Could not write uploaded package");
    }
    if fs::rename(&temporary, &destination).is_err() {
        let _ = fs::remove_file(&temporary);
        return error(StatusCode::INTERNAL_SERVER_ERROR, "Could not finalize uploaded package");
    }
    let size = bytes.len() as i64;
    let result = {
        let db = match state.db.lock() {
            Ok(v) => v,
            Err(_) => { let _ = fs::remove_file(&destination); return error(StatusCode::INTERNAL_SERVER_ERROR, "Database lock failed"); }
        };
        db.execute(
            "INSERT INTO apps (id,title,version,package_type,description,publisher,category,filename,sha256,size_bytes)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![id,title.trim(),version.trim(),package_type,description.trim(),publisher.trim(),category.trim(),stored_name,digest,size],
        )
    };
    if result.is_err() {
        let _ = fs::remove_file(&destination);
        return error(StatusCode::INTERNAL_SERVER_ERROR, "Could not save app metadata in SQLite");
    }
    Json(serde_json::json!({
        "id": id, "title": title.trim(), "version": version.trim(),
        "package_type": package_type, "filename": package_name, "sha256": digest,
        "size_bytes": size, "message": "Upload stored in App Hub catalogue"
    })).into_response()
}

async fn download_app(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    if !safe_component(&id, 140) { return error(StatusCode::BAD_REQUEST, "Invalid app ID"); }
    let filename: String = {
        let db = match state.db.lock() { Ok(v) => v, Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Database lock failed") };
        match db.query_row("SELECT filename FROM apps WHERE id=?1", params![id], |r| r.get(0)) {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => return error(StatusCode::NOT_FOUND, "App not found"),
            Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Database query failed"),
        }
    };
    if !safe_component(&filename, 220) { return error(StatusCode::INTERNAL_SERVER_ERROR, "Unsafe stored filename"); }
    let path = state.storage.join(&filename);
    let bytes = match tokio::fs::read(path).await {
        Ok(v) => v,
        Err(_) => return error(StatusCode::NOT_FOUND, "Package file is missing from storage"),
    };
    let content_type = if filename.ends_with(".lap") { "application/vnd.xzuff.lap" } else { "application/vnd.xzuff.luap" };
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response.headers_mut().insert(header::CONTENT_DISPOSITION, HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap());
    response
}

async fn health() -> &'static str { "xzuff-app-hub:ok" }

#[tokio::main]
async fn main() {
    let db_path = PathBuf::from(env::var("XZUFF_HUB_DB").unwrap_or_else(|_| "data/xzuff-app-hub.sqlite3".into()));
    let storage = PathBuf::from(env::var("XZUFF_HUB_STORAGE").unwrap_or_else(|_| "data/packages".into()));
    let bind: SocketAddr = env::var("XZUFF_HUB_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into())
        .parse().expect("XZUFF_HUB_BIND must be a socket address");
    if let Some(parent) = db_path.parent() { fs::create_dir_all(parent).expect("create database directory"); }
    fs::create_dir_all(&storage).expect("create package storage");
    let db = init_db(&db_path).expect("initialize SQLite schema");
    let state = AppState { db: Arc::new(Mutex::new(db)), storage: Arc::new(storage) };
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/apps", get(list_apps).post(upload_app))
        .route("/api/apps/:id/download", get(download_app))
        .layer(DefaultBodyLimit::max(MAX_PACKAGE_BYTES + 1024 * 1024))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind).await.expect("bind App Hub listener");
    println!("Xzuff App Hub listening on http://{}", bind);
    println!("SQLite: {}; package storage: {}", db_path.display(), state.storage.display());
    println!("Warning: upload endpoint has no authentication; keep bound to localhost or put behind authenticated HTTPS.");
    axum::serve(listener, app).await.expect("serve App Hub");
}
