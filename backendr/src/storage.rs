//! 本地磁盘存储层（file / file_transfer 模块）。
//!
//! 对齐 Go `pkg/oss`（MinIO）的 wire 语义：bucket 按 MIME 自动路由、文件内容
//! 嗅探真实 MIME（防伪造）、目录安全校验（防 `..` 穿越）、签名公开访问 URL
//! （HMAC-SHA256 + 有效期，走 `/admin/v1/file/image` 代理）。存储介质为本地磁盘
//! 而非 MinIO：`{upload_dir}/{bucket}/{object_key}`。
//!
//! 与 Go 的行为对齐点：
//! - `ContentTypeToBucketName` / `FileExtensionToBucketName`：images/videos/audios/docs/files；
//! - `DetectFileType`：魔术字节嗅探（png/jpg/gif/pdf/zip/mp4/mp3/wav/bmp）；
//! - `IsAllowedMimeType`：白名单（image/*/video/*/audio/*/text/* + pdf/office/zip 等）；
//! - `IsFileDirectorySafe`：仅字母数字_/-/斜杠、拒绝 `..` 与绝对路径；
//! - `SignData`：HMAC-SHA256(key=SHA-256(GOWIND_CRYPTO_KEY)) 十六进制签名，1 年有效期；
//! - `MaxUploadSize` / `MaxDownloadSize`：50 MiB。

use std::net::IpAddr;
use std::path::PathBuf;

use crate::error::AppError;

/// 单次上传最大字节数（对齐 Go oss.MaxUploadSize，50 MiB）
pub const MAX_UPLOAD_SIZE: usize = 50 * 1024 * 1024;
/// 外部 URL 下载最大字节数（对齐 Go oss.MaxDownloadSize，50 MiB）
pub const MAX_DOWNLOAD_SIZE: usize = 50 * 1024 * 1024;
/// 签名图片 URL 有效期：1 年（对齐 Go mediaURLTTL）
pub const MEDIA_URL_TTL_SECS: i64 = 365 * 24 * 3600;

// ===================== 桶路由（对齐 Go ContentTypeToBucketName） =====================

pub const BUCKET_IMAGES: &str = "images";
pub const BUCKET_VIDEOS: &str = "videos";
pub const BUCKET_AUDIOS: &str = "audios";
pub const BUCKET_DOCS: &str = "docs";
pub const BUCKET_FILES: &str = "files";

/// 按 MIME 主类型路由到桶（对齐 Go ContentTypeToBucketName）。
pub fn content_type_to_bucket(content_type: &str) -> &'static str {
    let ct = content_type.trim().to_ascii_lowercase();
    let (main, sub) = match ct.split_once('/') {
        Some((m, s)) => (m, s.split(';').next().unwrap_or(s).trim()),
        None => return BUCKET_FILES,
    };
    match main {
        "image" => BUCKET_IMAGES,
        "video" => BUCKET_VIDEOS,
        "audio" => BUCKET_AUDIOS,
        "text" => BUCKET_DOCS,
        "application" => match sub {
            "pdf" | "json" => BUCKET_DOCS,
            _ if sub.starts_with("vnd.ms-")
                || sub.contains("officedocument")
                || sub.contains("word")
                || sub.contains("excel")
                || sub.contains("powerpoint") =>
            {
                BUCKET_DOCS
            }
            _ => BUCKET_FILES,
        },
        _ => BUCKET_FILES,
    }
}

/// MIME → 文件扩展名（带前导点；未知返回空串，对齐 ContentTypeToFileExtension）。
pub fn mime_to_extension(content_type: &str) -> &'static str {
    match content_type.trim().to_ascii_lowercase().split(';').next().unwrap_or("").trim() {
        "image/jpeg" | "image/jpg" => ".jpg",
        "image/png" => ".png",
        "image/gif" => ".gif",
        "image/webp" => ".webp",
        "image/bmp" => ".bmp",
        "image/svg+xml" => ".svg",
        "image/x-icon" => ".ico",
        "video/mp4" => ".mp4",
        "video/webm" => ".webm",
        "audio/mpeg" => ".mp3",
        "audio/wav" => ".wav",
        "audio/ogg" => ".ogg",
        "application/pdf" => ".pdf",
        "application/json" => ".json",
        "application/zip" => ".zip",
        "text/plain" => ".txt",
        "text/markdown" => ".md",
        "text/html" | "text/css" | "application/javascript" | "text/javascript" => ".txt",
        _ => "",
    }
}

/// 忽略参数的 MIME 主类型（如 "text/plain; charset=utf-8" → "text/plain"）。
pub fn plain_mime(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_string()
}

/// 扩展名 → Content-Type（serve_image 回源时用；未知默认 octet-stream）。
pub fn content_type_for_extension(ext: &str) -> &'static str {
    let e = ext.trim().trim_start_matches('.').to_ascii_lowercase();
    match e.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "heic" => "image/heic",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "pdf" => "application/pdf",
        "json" => "application/json",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "csv" => "text/csv",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}

// ===================== MIME 白名单（对齐 Go IsAllowedMimeType） =====================

/// 上传 MIME 白名单：主类型前缀 + 精确类型集合（pdf/office/zip 等）。
pub fn is_allowed_mime(mime_type: &str) -> bool {
    let mime = mime_type.trim().to_ascii_lowercase();
    if mime.is_empty() {
        return false;
    }
    match mime.as_str() {
        "application/pdf" | "application/json" | "application/zip" | "application/x-rar-compressed"
        | "application/msword"
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        | "application/vnd.ms-excel"
        | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        | "application/vnd.ms-powerpoint"
        | "application/vnd.openxmlformats-officedocument.presentationml.presentation" => true,
        main if main.starts_with("image/")
            || main.starts_with("video/")
            || main.starts_with("audio/")
            || main.starts_with("text/") =>
        {
            true
        }
        _ => false,
    }
}

/// 目录安全校验（对齐 Go IsFileDirectorySafe）：仅字母/数字/_/-/斜杠，拒绝 `..` 与绝对路径。
pub fn is_directory_safe(dir: &str) -> bool {
    if dir.is_empty() {
        return true;
    }
    if dir.contains("..") || dir.starts_with('/') || dir.starts_with('\\') {
        return false;
    }
    dir.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/'))
}

// ===================== MIME 嗅探（对齐 Go DetectFileType） =====================

/// 基于文件内容魔术字节嗅探真实 MIME 与扩展名（带前导点）。
pub fn detect_file_type(data: &[u8]) -> (String, String) {
    let head = &data[..data.len().min(512)];
    if data.len() >= 8 && &data[..8] == b"\x89PNG\r\n\x1a\n" {
        return ("image/png".into(), ".png".into());
    }
    if data.len() >= 3 && data[..3] == [0xff, 0xd8, 0xff] {
        return ("image/jpeg".into(), ".jpg".into());
    }
    if data.len() >= 6 && (&data[..6] == b"GIF87a" || &data[..6] == b"GIF89a") {
        return ("image/gif".into(), ".gif".into());
    }
    if data.len() >= 5 && &data[..5] == b"%PDF-" {
        return ("application/pdf".into(), ".pdf".into());
    }
    if data.len() >= 4 && &data[..4] == b"PK\x03\x04" {
        return ("application/zip".into(), ".zip".into());
    }
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        return ("video/mp4".into(), ".mp4".into());
    }
    if data.len() >= 3 && &data[..3] == b"ID3" {
        return ("audio/mpeg".into(), ".mp3".into());
    }
    if data.len() >= 2 && data[0] == 0xff && (data[1] & 0xe0) == 0xe0 {
        return ("audio/mpeg".into(), ".mp3".into());
    }
    if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WAVE" {
        return ("audio/wav".into(), ".wav".into());
    }
    if data.len() >= 2 && &data[..2] == b"BM" {
        return ("image/bmp".into(), ".bmp".into());
    }
    // WebP 魔数（RIFF....WEBP）
    if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return ("image/webp".into(), ".webp".into());
    }
    if data.len() >= 4 && &data[..4] == b"OggS" {
        return ("audio/ogg".into(), ".ogg".into());
    }
    if data.len() >= 4 && &data[..4] == b"\x1a\x45\xdf\xa3" {
        return ("video/webm".into(), ".webm".into());
    }
    // UTF-8/ASCII 文本兜底（避免把纯文本误判成无关类型）
    if head.is_ascii() && !head.is_empty() {
        return ("text/plain".into(), ".txt".into());
    }
    (String::new(), String::new())
}

// ===================== 对象路径工具（对齐 Go joinObjectPath / parseKey） =====================

/// 拼接对象 key：目录与文件名之间补 `/`，目录为空或文件名已含前导 `/` 时避免重复斜杠。
pub fn join_object_path(directory: &str, file_name: &str) -> String {
    let dir = directory.trim_matches('/');
    let name = file_name.trim_start_matches('/');
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

/// 解析对象 key 为 (目录, 文件名, 扩展名无点)。目录 key 以 `/` 结尾时文件名段落为空。
pub fn parse_key(key: &str) -> (String, String, String) {
    if key.is_empty() {
        return (String::new(), String::new(), String::new());
    }
    let key = key.trim_start_matches('/');
    if key.ends_with('/') {
        return (key.trim_end_matches('/').to_string(), String::new(), String::new());
    }
    let dir = match key.rsplit_once('/') {
        Some((d, _)) if !d.is_empty() => d.to_string(),
        _ => String::new(),
    };
    let base = key.rsplit('/').next().unwrap_or(key).to_string();
    // 点文件（如 .env）：仅当前有一个前导点且无其他点 → 无扩展名
    if base.starts_with('.') && base.matches('.').count() == 1 {
        return (dir, base, String::new());
    }
    match base.rfind('.') {
        Some(idx) if idx > 0 => (
            dir,
            base[..idx].to_string(),
            base[idx + 1..].to_ascii_lowercase(),
        ),
        _ => (dir, base, String::new()),
    }
}

// ===================== 对象命名（对齐 Go GenerateObjectName UUID 模式） =====================

/// UUID 文件名生成（Go 端 id.NewGUIDv7(false)：去横线小写）。
fn uuid_file_name() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

// ===================== 本地存储实现 =====================

#[derive(Debug, Clone)]
pub struct LocalStorage {
    /// 上传根目录（`GW_ADMIN_UPLOAD_DIR`，默认 `./uploads`）
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SavedFile {
    pub bucket: String,
    /// 对象 key（含目录，如 `user/1001/xxx.jpg`）
    pub object_key: String,
    /// 存储路径（Go storagePath：`/bucket/object`，签名 URL 用）
    pub storage_path: String,
    /// 可下载的对象引用（Go downloadUrl 语义：`{bucket}/{object}`）
    pub download_url: String,
}

#[derive(Debug, Clone)]
pub struct LoadedFile {
    pub data: Vec<u8>,
    pub mime: String,
    pub size: i64,
    /// SHA-256 十六进制（小写）
    pub sha256: String,
}

impl LocalStorage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// 落盘：`{root}/{bucket}/{object_key}`（自动建目录）。
    pub fn save(
        &self,
        bucket: &str,
        directory: &str,
        source_file_name: &str,
        mime: &str,
        data: &[u8],
    ) -> Result<SavedFile, AppError> {
        if data.is_empty() {
            return Err(AppError::Validation("empty file data".into()));
        }
        if data.len() > MAX_UPLOAD_SIZE {
            return Err(AppError::Validation(format!(
                "file size {} exceeds max upload size {}",
                data.len(),
                MAX_UPLOAD_SIZE
            )));
        }
        if mime.trim().is_empty() {
            return Err(AppError::Validation("unknown mime type".into()));
        }
        // source_file_name 为空合法（对齐 Go UploadFile：对象名由 UUID 自动生成，如 avatar 场景）；
        // 协议层必填校验由 file_transfer handle_upload 负责（`unknown source file name`）。
        if !is_directory_safe(directory) {
            return Err(AppError::Validation("invalid file directory".into()));
        }

        let (sniffed_mime, ext) = detect_file_type(data);
        // 嗅探出的真实类型覆盖客户端声明（防 bucket 路由被绕过）
        let mime = if sniffed_mime.is_empty() {
            plain_mime(mime)
        } else {
            sniffed_mime
        };
        let ext = if ext.is_empty() {
            mime_to_extension(&mime).to_string()
        } else {
            ext
        };
        if !is_allowed_mime(&mime) {
            return Err(AppError::Validation(format!(
                "file type \"{mime}\" is not allowed"
            )));
        }

        let bucket = if bucket.trim().is_empty() {
            content_type_to_bucket(&mime).to_string()
        } else {
            bucket.trim().to_string()
        };
        let obj_name = format!("{}{}", uuid_file_name(), ext);
        // 目录规范化：去首尾斜杠，拒绝穿越（is_directory_safe 已保证）
        let dir = directory.trim_matches('/').to_string();
        let object_key = join_object_path(&dir, &obj_name);

        let full = self.full_path(&bucket, &object_key)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Internal {
                context: format!("create upload dir {} failed", parent.display()),
                source: Some(Box::new(e)),
            })?;
        }
        std::fs::write(&full, data).map_err(|e| AppError::Internal {
            context: format!("write upload file {} failed", full.display()),
            source: Some(Box::new(e)),
        })?;

        let storage_path = format!("/{bucket}/{object_key}");
        let download_url = format!("{bucket}/{object_key}");
        Ok(SavedFile {
            bucket,
            object_key,
            storage_path,
            download_url,
        })
    }

    /// 读盘：返回字节、MIME、大小与 SHA-256。路径穿越在 full_path 处统一拒绝。
    pub fn load(&self, bucket: &str, object_key: &str) -> Result<LoadedFile, AppError> {
        if bucket.trim().is_empty() || object_key.trim().is_empty() {
            return Err(AppError::Validation("invalid storage object".into()));
        }
        let full = self.full_path(bucket, object_key)?;
        let full = std::fs::read(&full).map_err(|_e| AppError::NotFound("file not found".into()))?;
        if full.len() > MAX_DOWNLOAD_SIZE {
            return Err(AppError::NotFound("file too large to download".into()));
        }
        let (_, _, ext) = parse_key(object_key);
        let mime = if ext.is_empty() {
            "application/octet-stream".into()
        } else {
            content_type_for_extension(&ext).to_string()
        };
        let sha256 = {
            use sha2::{Digest, Sha256};
            hex::encode(Sha256::digest(&full))
        };
        let size = data_len(&full)?;
        Ok(LoadedFile {
            data: full,
            mime,
            size,
            sha256,
        })
    }

    /// 删除对象（不存在视为成功，对齐 Go 幂等删除）。
    pub fn delete(&self, bucket: &str, object_key: &str) -> Result<(), AppError> {
        let full = self.full_path(bucket, object_key)?;
        match std::fs::remove_file(&full) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AppError::Internal {
                context: format!("delete file {} failed", full.display()),
                source: Some(Box::new(e)),
            }),
        }
    }

    fn full_path(&self, bucket: &str, object_key: &str) -> Result<PathBuf, AppError> {
        for seg in object_key.split('/') {
            if seg.is_empty() || seg == "." || seg == ".." || seg.contains('\\')
                || seg.contains(':')
            {
                return Err(AppError::Validation("invalid object path".into()));
            }
        }
        let mut p = self.root.clone();
        p.push(bucket.trim_matches('/'));
        p.push(object_key.trim_matches('/'));
        Ok(p)
    }
}

fn data_len(data: &[u8]) -> Result<i64, AppError> {
    i64::try_from(data.len()).map_err(|_| AppError::Internal {
        context: "file size overflow".into(),
        source: None,
    })
}

// ===================== 文件大小格式化（对齐 Go FileRepo.formatSize：1024 进制） =====================

/// 字节数 → 人类可读大小（如 `512B` / `2.34MB` / `1.1GB`）。
pub fn format_size(size: i64) -> String {
    if size <= 0 {
        return "0B".into();
    }
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut s = size as f64;
    let mut i = 0usize;
    while s >= 1024.0 && i < units.len() - 1 {
        s /= 1024.0;
        i += 1;
    }
    if i == 0 {
        return format!("{size}B");
    }
    let v = (s * 100.0).round() / 100.0;
    let mut str = format!("{v:.2}");
    while str.ends_with('0') {
        str.pop();
    }
    if str.ends_with('.') {
        str.pop();
    }
    format!("{str}{}", units[i])
}

// ===================== 签名公开访问 URL（对齐 Go SignData + ServeImageHandler） =====================

/// 生成签名媒体 URL（`/admin/v1/file/image?path=...&expires=...&sig=...`）。
/// `GOWIND_CRYPTO_KEY` 未配置时返回空串（对齐 Go：签名不可用则前端无法内嵌预览）。
pub fn signed_media_url(storage_path: &str) -> String {
    let expires = (chrono::Utc::now().timestamp() + MEDIA_URL_TTL_SECS).to_string();
    let data = format!("{storage_path}|{expires}");
    match crate::crypto::sign_data(&data) {
        Some(sig) => {
            let path = percent_encoding::percent_encode(
                storage_path.as_bytes(),
                percent_encoding::NON_ALPHANUMERIC,
            )
            .to_string();
            format!("/admin/v1/file/image?path={path}&expires={expires}&sig={sig}")
        }
        None => String::new(),
    }
}

/// 生成签名媒体 URL，有效期可自定义（`presignExpireSeconds`；对齐 Go presign 语义）。
/// `GOWIND_CRYPTO_KEY`/`GO_WIND_CRYPTO_KEY` 未配置或生成失败时返回空串。
pub fn signed_media_url_with_ttl(storage_path: &str, ttl_secs: i64) -> String {
    let expires = (chrono::Utc::now().timestamp() + ttl_secs.max(1)).to_string();
    let data = format!("{storage_path}|{expires}");
    match crate::crypto::sign_data(&data) {
        Some(sig) => {
            let path = percent_encoding::percent_encode(
                storage_path.as_bytes(),
                percent_encoding::NON_ALPHANUMERIC,
            )
            .to_string();
            format!("/admin/v1/file/image?path={path}&expires={expires}&sig={sig}")
        }
        None => String::new(),
    }
}

/// 校验签名媒体 URL 参数（`path|expires` 的 HMAC + 过期时间）。
pub fn verify_media_url(path: &str, expires: &str, sig: &str) -> bool {
    if path.is_empty() || expires.is_empty() || sig.is_empty() {
        return false;
    }
    let Ok(expires_ts) = expires.trim().parse::<i64>() else {
        return false;
    };
    if chrono::Utc::now().timestamp() > expires_ts {
        return false;
    }
    crate::crypto::verify_signature(&format!("{path}|{expires}"), sig)
}

// ===================== SSRF 防护下载（对齐 Go FileTransferService.downloadFileFromURL） =====================

/// 判断 IP 是否应被阻断（私网/环回/链路本地/多播/未指定等）。对齐 Go netutil.LookupAndCheckHost。
fn is_blocked_ip(ip: IpAddr) -> bool {
    use std::net::Ipv4Addr;
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.is_documentation()
                || v4.octets()[0] == 0
                || (v4.octets()[0] == 100 && (64..=127).contains(&v4.octets()[1])) // CGNAT 100.64/10
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 ULA
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
                || v6.is_unique_local()
        }
    }
}

/// 静态校验下载 URL：scheme http/https、无 userinfo、host 非空。
pub fn validate_download_url(raw: &str) -> Result<url::Url, AppError> {
    let u = url::Url::parse(raw).map_err(|e| {
        AppError::Validation(format!("invalid download url: {e}"))
    })?;
    if u.scheme() != "http" && u.scheme() != "https" {
        return Err(AppError::Validation(format!(
            "invalid download url scheme: {}",
            u.scheme()
        )));
    }
    if !u.username().is_empty() || u.password().is_some() {
        return Err(AppError::Validation("invalid download url userinfo".into()));
    }
    if u.host_str().is_none() {
        return Err(AppError::Validation("invalid download url host".into()));
    }
    Ok(u)
}

/// 解析主机并校验所有解析出的 IP 均不在内网；返回 (host, 首选合法 IP)。
/// 失败统一按"被阻断"处理（对齐 Go LookupAndCheckHost 的 fail-closed 语义）。
pub async fn lookup_safe_host(
    host: &str,
    port: u16,
) -> Result<(String, IpAddr), AppError> {
    let addr = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| AppError::Forbidden(format!("blocked download host: {e}")))?;
    let mut first: Option<IpAddr> = None;
    let mut dedup = std::collections::HashSet::new();
    for sa in addr {
        let ip = sa.ip();
        if is_blocked_ip(ip) {
            continue;
        }
        if dedup.insert(ip) && first.is_none() {
            first = Some(ip);
        }
    }
    let Some(pinned) = first else {
        return Err(AppError::Forbidden("blocked download host: no public ip".into()));
    };
    Ok((host.to_string(), pinned))
}

/// 从外部 URL 下载（SSRF 防护：scheme 白名单、解析逐 IP 校验、pin IP 防 DNS rebinding、
/// 手动重定向循环逐跳重新校验、响应体大小限制、超时）。对齐 Go downloadFileFromURL。
pub async fn download_from_url(raw: &str) -> Result<Vec<u8>, AppError> {
    let u = validate_download_url(raw)?;

    let mut current = u;
    let mut hops = 0usize;
    loop {
        // 每一跳都重新做静态校验 + DNS 校验 + pin IP（对齐 Go CheckRedirect 逐跳校验）
        let chost = current.host_str().unwrap_or_default().to_string();
        let cport = current.port_or_known_default().unwrap_or(80);
        let (_, pinned) = lookup_safe_host(&chost, cport).await?;

        // reqwest 0.12 的 resolve 挂在 ClientBuilder 上：按跳新建 client 并固定解析到已验证 IP
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none()) // 手动跟随，逐跳校验
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30))
            .resolve(&chost, std::net::SocketAddr::new(pinned, cport))
            .build()
            .map_err(|e| AppError::Internal {
                context: format!("build download client failed: {e}"),
                source: None,
            })?;

        let resp = client
            .get(current.as_str())
            .send()
            .await
            .map_err(|e| AppError::Validation(format!("download failed: {e}")))?;

        let resp = if resp.status().is_redirection() {
            if hops >= 5 {
                return Err(AppError::Validation("download failed: too many redirects".into()));
            }
            let next = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| AppError::Validation("download failed: redirect without location".into()))?;
            let joined = current.join(next).map_err(|e| {
                AppError::Validation(format!("download failed: invalid redirect: {e}"))
            })?;
            hops += 1;
            current = joined;
            continue;
        } else if resp.status().is_success() {
            resp
        } else {
            return Err(AppError::Validation(format!(
                "download failed: unexpected status: {}",
                resp.status()
            )));
        };

        // 限制响应体大小
        let limited = resp.bytes().await.map_err(|e| {
            AppError::Validation(format!("download failed: read body failed: {e}"))
        })?;
        if limited.len() > MAX_DOWNLOAD_SIZE {
            return Err(AppError::Validation(format!(
                "download failed: remote file exceeds max download size {MAX_DOWNLOAD_SIZE}"
            )));
        }
        return Ok(limited.to_vec());
    }
}