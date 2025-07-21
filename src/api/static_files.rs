use anyhow::Result;
use hyper::{Body, Response, StatusCode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct StaticOptions {
    pub dotfiles: String,        // "allow", "deny", "ignore"
    pub etag: bool,              // Enable/disable etag generation
    pub extensions: Vec<String>, // File extensions to try
    pub index: Option<String>,   // Index file name, None to disable
    pub max_age: u64,            // Cache max-age in seconds
    pub redirect: bool,          // Redirect trailing slash
    pub set_headers: bool,       // Allow custom headers
}

impl Default for StaticOptions {
    fn default() -> Self {
        Self {
            dotfiles: "ignore".to_string(),
            etag: true,
            extensions: vec!["html".to_string()],
            index: Some("index.html".to_string()),
            max_age: 0,
            redirect: true,
            set_headers: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaticMiddleware {
    pub root_dir: PathBuf,
    pub options: StaticOptions,
}

impl StaticMiddleware {
    pub fn new(root: &str, options: Option<StaticOptions>) -> Self {
        Self {
            root_dir: PathBuf::from(root),
            options: options.unwrap_or_default(),
        }
    }

    pub async fn serve_file(&self, request_path: &str) -> Result<Response<Body>> {
        let safe_path = self.sanitize_path(request_path)?;
        let full_path = self.root_dir.join(&safe_path);

        // Security check - ensure path is within root directory
        if !full_path.starts_with(&self.root_dir) {
            return self.forbidden_response().await;
        }

        // Check if path exists
        if !full_path.exists() {
            return self.try_with_extensions(&full_path).await;
        }

        // Handle directory requests
        if full_path.is_dir() {
            return self.serve_directory(&full_path).await;
        }

        // Check dotfiles policy
        if self.is_dotfile(&safe_path) && self.options.dotfiles == "deny" {
            return self.forbidden_response().await;
        }

        if self.is_dotfile(&safe_path) && self.options.dotfiles == "ignore" {
            return self.not_found_response().await;
        }

        // Serve the file
        self.serve_static_file(&full_path).await
    }

    fn sanitize_path(&self, path: &str) -> Result<String> {
        // Remove leading slash and resolve relative paths
        let clean_path = path.trim_start_matches('/');

        // Prevent directory traversal
        let path_buf = PathBuf::from(clean_path);
        let mut safe_components = Vec::new();

        for component in path_buf.components() {
            match component {
                std::path::Component::Normal(name) => {
                    safe_components.push(name.to_string_lossy().to_string());
                }
                std::path::Component::ParentDir => {
                    // Ignore parent directory traversal attempts
                    continue;
                }
                _ => continue,
            }
        }

        Ok(safe_components.join("/"))
    }

    fn is_dotfile(&self, path: &str) -> bool {
        path.split('/').any(|component| component.starts_with('.'))
    }

    async fn try_with_extensions(&self, base_path: &Path) -> Result<Response<Body>> {
        for ext in &self.options.extensions {
            let path_with_ext = base_path.with_extension(ext);
            if path_with_ext.exists() && path_with_ext.is_file() {
                return self.serve_static_file(&path_with_ext).await;
            }
        }

        self.not_found_response().await
    }

    async fn serve_directory(&self, dir_path: &Path) -> Result<Response<Body>> {
        if let Some(ref index_file) = self.options.index {
            let index_path = dir_path.join(index_file);
            if index_path.exists() && index_path.is_file() {
                return self.serve_static_file(&index_path).await;
            }
        }

        // Directory listing disabled by default for security
        self.forbidden_response().await
    }

    async fn serve_static_file(&self, file_path: &Path) -> Result<Response<Body>> {
        match fs::read(file_path).await {
            Ok(contents) => {
                let mime_type = self.get_mime_type(file_path);
                let mut response = Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", mime_type);

                // Add caching headers
                if self.options.max_age > 0 {
                    response = response
                        .header("cache-control", format!("max-age={}", self.options.max_age));
                }

                // Add ETag if enabled
                if self.options.etag {
                    let etag = self.generate_etag(&contents);
                    response = response.header("etag", etag);
                }

                Ok(response.body(Body::from(contents)).unwrap())
            }
            Err(_) => self.not_found_response().await,
        }
    }

    fn get_mime_type(&self, file_path: &Path) -> &'static str {
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("html") | Some("htm") => "text/html; charset=utf-8",
            Some("css") => "text/css",
            Some("js") | Some("mjs") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("webp") => "image/webp",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            Some("eot") => "application/vnd.ms-fontobject",
            Some("pdf") => "application/pdf",
            Some("txt") => "text/plain; charset=utf-8",
            Some("xml") => "application/xml",
            Some("zip") => "application/zip",
            Some("tar") => "application/x-tar",
            Some("gz") => "application/gzip",
            Some("mp4") => "video/mp4",
            Some("webm") => "video/webm",
            Some("mp3") => "audio/mpeg",
            Some("wav") => "audio/wav",
            Some("ogg") => "audio/ogg",
            _ => "application/octet-stream",
        }
    }

    fn generate_etag(&self, contents: &[u8]) -> String {
        // Simple ETag generation using content length and basic hash
        let hash = contents.len();
        format!("\"{}\"", hash)
    }

    async fn not_found_response(&self) -> Result<Response<Body>> {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/plain")
            .body(Body::from("File not found"))
            .unwrap())
    }

    async fn forbidden_response(&self) -> Result<Response<Body>> {
        Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("content-type", "text/plain")
            .body(Body::from("Forbidden"))
            .unwrap())
    }
}

/// Create a static file middleware function
pub fn create_static_middleware(root: &str, options: Option<StaticOptions>) -> StaticMiddleware {
    StaticMiddleware::new(root, options)
}

/// Check if a request path should be handled by static middleware
pub fn should_handle_static(path: &str, mount_path: &str) -> bool {
    if mount_path == "/" || mount_path.is_empty() {
        return true;
    }

    path.starts_with(mount_path)
}

/// Strip mount path from request path
pub fn strip_mount_path(path: &str, mount_path: &str) -> String {
    if mount_path == "/" || mount_path.is_empty() {
        return path.to_string();
    }

    path.strip_prefix(mount_path)
        .unwrap_or(path)
        .trim_start_matches('/')
        .to_string()
}
