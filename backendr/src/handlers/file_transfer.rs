// file_transfer 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、uint64 字符串化）。
// 对齐 Go file_transfer_service.go：
//  - POST/PUT /file/upload：multipart 表单（file/storageObject/sourceFileName/mime/size）→
//    嗅探真实 MIME（覆盖客户端声明）→ 目录安全校验 → 本地磁盘落盘 → files 表落元数据 →
//    `{objectName, presignedUrl, publicUrl}`（objectName=Go 的 downloadUrl 语义）；
//  - GET /file/download：storageObject（bucket+object）直读或签名 URL；fileId → 查 files 表
//    元数据转 storageObject 同链路（对齐 Go）；downloadUrl → SSRF 防护代理下载；
//  - GET /file/image：HMAC 签名 + 有效期校验后回源流式返回（免鉴权，签名即凭证）。

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::handlers::file::local_storage;
use crate::middleware::Operator;
use crate::repos::file::FileRepo;
use crate::state::AppState;
use crate::storage;

// ===================== 请求/响应 DTO（对齐 proto protojson 字段名） =====================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileResponse {
    /// Go 端将此字段置为 downloadUrl（MinIO 公开地址）；本端为 `{bucket}/{object}` 相对引用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_name: Option<String>,
    /// 预签名直传路径恒为 null（Go 端同语义：presigned upload 已禁用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presigned_url: Option<String>,
    /// 签名公开访问 URL（`/admin/v1/file/image?...`）；GOWIND_CRYPTO_KEY 未配置时省略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadFileResponse {
    /// 直接返回文件字节（base64；protojson bytes → base64 字符串）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// 预签名/签名 URL（preferPresignedUrl=true 时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadStorageObject {
    #[serde(default)]
    pub bucket_name: Option<String>,
    #[serde(default)]
    pub file_directory: Option<String>,
    #[serde(default)]
    pub object_name: Option<String>,
}

// ===================== 上传（POST/PUT /file/upload，multipart） =====================

async fn handle_upload(
    state: &AppState,
    operator: &Operator,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<UploadFileResponse>, AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut storage_object: Option<UploadStorageObject> = None;
    let mut source_file_name = String::new();
    let mut mime = String::new();
    let mut size_hint: Option<i64> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart parse failed: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let bytes = field.bytes().await.map_err(|e| {
                    AppError::Validation(format!("read file field failed: {e}"))
                })?;
                file_data = Some(bytes.to_vec());
            }
            "storageObject" => {
                if let Ok(text) = field.text().await {
                    if !text.trim().is_empty() {
                        storage_object = serde_json::from_str(&text)
                            .map_err(|e| AppError::Validation(format!("invalid storageObject: {e}")))?;
                    }
                }
            }
            "sourceFileName" => {
                source_file_name = field.text().await.unwrap_or_default();
            }
            "mime" => {
                mime = field.text().await.unwrap_or_default();
            }
            "size" => {
                if let Ok(text) = field.text().await {
                    size_hint = text.trim().parse().ok();
                }
            }
            _ => {
                // 忽略未知字段（前端兼容）
                let _ = field.bytes().await;
            }
        }
    }

    let storage_object = storage_object.ok_or_else(|| AppError::Validation("unknown storageObject".into()))?;
    let file_data = file_data.ok_or_else(|| AppError::Validation("unknown fileData".into()))?;
    if mime.trim().is_empty() {
        return Err(AppError::Validation("unknown mime type".into()));
    }
    if source_file_name.trim().is_empty() {
        return Err(AppError::Validation("unknown source file name".into()));
    }
    let _ = size_hint; // 服务端以真实字节数为准

    let storage = local_storage(state);
    let saved = storage.save(
        storage_object.bucket_name.as_deref().unwrap_or(""),
        storage_object.file_directory.as_deref().unwrap_or(""),
        &source_file_name,
        &mime,
        &file_data,
    )?;

    // 记录文件元数据（对齐 Go recordFile）
    let sha256 = {
        use sha2::{Digest, Sha256};
        hex::encode(Sha256::digest(&file_data))
    };
    let dir = storage_object.file_directory.as_deref().unwrap_or("").trim_matches('/').to_string();
    let (_, obj_name, ext) = storage::parse_key(&saved.object_key);
    let file_guid = uuid::Uuid::new_v4().simple().to_string();
    let size = i64::try_from(file_data.len()).map_err(|_| AppError::Internal {
        context: "file size overflow".into(),
        source: None,
    })?;

    let db = crate::handlers::script::db_of(state)?;
    let repo = FileRepo::new(db);
    repo.create(
        "MINIO", // 对齐 Go recordFile：provider=MINIO
        Some(&saved.bucket),
        Some(&dir),
        Some(&file_guid),
        Some(&format!("{obj_name}.{ext}")),
        Some(&source_file_name),
        Some(&ext),
        Some(size),
        Some(&storage::format_size(size)),
        Some(&saved.download_url),
        Some(&sha256),
        Some(operator.tenant_id),
        operator.user_id,
    )
    .await?;

    // 生成签名公开访问 URL（1 年有效期）；未配置 GOWIND_CRYPTO_KEY 时返回空串（对齐 Go）
    let public_url = storage::signed_media_url(&saved.storage_path);

    Ok(Json(UploadFileResponse {
        object_name: Some(saved.download_url),
        presigned_url: None,
        public_url: if public_url.is_empty() {
            None
        } else {
            Some(public_url)
        },
    }))
}

pub async fn file_transfer_put_upload_file(
    State(state): State<AppState>,
    operator: Operator,
    multipart: axum::extract::Multipart,
) -> Result<impl IntoResponse, AppError> {
    handle_upload(&state, &operator, multipart).await
}

pub async fn file_transfer_post_upload_file(
    State(state): State<AppState>,
    operator: Operator,
    multipart: axum::extract::Multipart,
) -> Result<impl IntoResponse, AppError> {
    handle_upload(&state, &operator, multipart).await
}

// ===================== 下载（GET /file/download） =====================

pub async fn file_transfer_download_file(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let prefer_presigned = params
        .get("preferPresignedUrl")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let presign_expire_seconds = params
        .get("presignExpireSeconds")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(storage::MEDIA_URL_TTL_SECS);
    let accept_mime = params.get("acceptMime").cloned().unwrap_or_default();
    let disposition = params.get("disposition").cloned().unwrap_or_default();
    let _ = disposition;

    // selector 1: downloadUrl（外部 URL，SSRF 防护代理）
    if let Some(download_url) = params.get("downloadUrl").filter(|v| !v.trim().is_empty()) {
        let data = storage::download_from_url(download_url).await?;
        let mime = storage::detect_file_type(&data).0;
        return Ok(Json(download_file_response(
            data,
            download_url.rsplit('/').next().unwrap_or("download").into(),
            mime,
        )));
    }

    // selector 2: storageObject（bucket + object）
    if let Some(bucket) = params.get("storageObject.bucketName").filter(|v| !v.trim().is_empty()) {
        let object = params
            .get("storageObject.objectName")
            .cloned()
            .or_else(|| {
                params.get("storageObject.fileDirectory").map(|d| {
                    storage::join_object_path(d, params.get("sourceFileName").map(String::as_str).unwrap_or(""))
                })
            })
            .unwrap_or_default();

        return download_by_storage_object(
            &state,
            &bucket,
            &object,
            prefer_presigned,
            presign_expire_seconds,
            &accept_mime,
        );
    }

    // selector 3: fileId — 查 files 表元数据转 storageObject 后走同一下载链路（对齐 Go DownloadFile）
    if let Some(file_id_raw) = params.get("fileId").filter(|v| !v.trim().is_empty()) {
        let file_id: i64 = file_id_raw
            .trim()
            .parse()
            .map_err(|_| AppError::Validation("invalid fileId".into()))?;
        let db = crate::handlers::script::db_of(&state)?;
        let row = FileRepo::new(db)
            .get(file_id)
            .await?
            .ok_or_else(|| AppError::NotFound("file not found".into()))?;
        let bucket = row.bucket_name.unwrap_or_default();
        let object = storage::join_object_path(
            row.file_directory.as_deref().unwrap_or(""),
            row.save_file_name.as_deref().unwrap_or(""),
        );
        if bucket.is_empty() || object.is_empty() {
            return Err(AppError::NotFound("file not found".into()));
        }
        return download_by_storage_object(
            &state,
            &bucket,
            &object,
            prefer_presigned,
            presign_expire_seconds,
            &accept_mime,
        );
    }

    Err(AppError::Validation("unknown download selector".into()))
}

fn download_by_storage_object(
    state: &AppState,
    bucket: &str,
    object: &str,
    prefer_presigned: bool,
    presign_expire_seconds: i64,
    accept_mime: &str,
) -> Result<Json<DownloadFileResponse>, AppError> {
    let storage = local_storage(state);
    if prefer_presigned {
        // 优先返回签名 URL（新标签页直开可用）；GOWIND_CRYPTO_KEY 未配置时降级直读
        let storage_path = format!("/{bucket}/{object}");
        let url = storage::signed_media_url_with_ttl(&storage_path, presign_expire_seconds);
        if !url.is_empty() {
            return Ok(Json(DownloadFileResponse {
                file: None,
                download_url: Some(url),
                source_file_name: None,
                mime: Some(Default::default()),
                size: Some(0),
                checksum: None,
                storage_path: Some(storage_path),
                updated_at: None,
            }));
        }
    }
    let loaded = storage.load(bucket, object)?;
    let source_name = object.rsplit('/').next().unwrap_or("download").to_string();
    let mime = if accept_mime.is_empty() {
        loaded.mime
    } else {
        accept_mime.to_string()
    };
    Ok(Json(download_file_response(loaded.data, source_name, mime)))
}

fn download_file_response(data: Vec<u8>, source_file_name: String, mime: String) -> DownloadFileResponse {
    let checksum = {
        use sha2::{Digest, Sha256};
        hex::encode(Sha256::digest(&data))
    };
    let size = i64::try_from(data.len()).unwrap_or(0);
    DownloadFileResponse {
        file: Some(base64::engine::general_purpose::STANDARD.encode(&data)),
        download_url: None,
        source_file_name: Some(source_file_name),
        mime: Some(if mime.is_empty() {
            "application/octet-stream".into()
        } else {
            mime
        }),
        size: Some(size),
        checksum: Some(checksum),
        storage_path: None,
        updated_at: None,
    }
}

// ===================== 签名图片代理（GET /file/image） =====================

/// GET /admin/v1/file/image?path={bucket}/{object}&expires={unix}&sig={hmac}
/// 免鉴权（签名即凭证），校验通过后回源本地磁盘，带 Cache-Control immutable。
pub async fn file_transfer_serve_image(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    let path = params.get("path").cloned().unwrap_or_default();
    let expires = params.get("expires").cloned().unwrap_or_default();
    let sig = params.get("sig").cloned().unwrap_or_default();

    if !storage::verify_media_url(&path, &expires, &sig) {
        if path.is_empty() || expires.is_empty() || sig.is_empty() {
            return Err(AppError::Validation("missing parameters".into()));
        }
        return Err(AppError::Forbidden("invalid signature".into()));
    }

    // storagePath 形如 /bucket/object（去掉前导斜杠后拆两段）
    let media_path = path.trim_start_matches('/');
    let Some((bucket, object)) = media_path.split_once('/') else {
        return Err(AppError::Validation("invalid path".into()));
    };
    if bucket.is_empty() || object.is_empty() {
        return Err(AppError::Validation("invalid path".into()));
    }

    let storage = local_storage(&state);
    let loaded = storage.load(bucket, object)?;

    let mut resp = Response::new(Body::from(loaded.data));
    resp.headers_mut()
        .insert(CONTENT_TYPE, loaded.mime.parse().unwrap_or_else(|_| "application/octet-stream".parse().unwrap()));
    resp.headers_mut()
        .insert(CONTENT_LENGTH, loaded.size.to_string().parse().unwrap_or_else(|_| "0".parse().unwrap()));
    resp.headers_mut()
        .insert(CACHE_CONTROL, "public, max-age=31536000, immutable".parse().unwrap());
    Ok(resp)
}