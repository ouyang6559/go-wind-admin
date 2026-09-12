// file 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、uint64 字符串化）。
// 对齐 Go file_service.go / data/file_repo.go：
//  - List：分页/过滤 + LEFT JOIN sys_tenants 回填 tenantName；
//  - Get：id 查询；
//  - Create/Update：`{data}` 包裹；size 写入时自动算 sizeFormat（1024 进制）；
//    Update 支持 allowMissing→Create（对齐 Go Update 的 allowMissing 分支）；
//  - Delete：硬删除（Go FileService.Delete 还同步删 OSS 对象；本端本地磁盘由事务内同步删除）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::file::{FileRepo, FileRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;
use crate::storage;

/// 可过滤/排序的白名单列（对应 files 表）
const FILE_COLUMNS: &[&str] = &[
    "id", "provider", "bucket_name", "file_directory", "file_guid", "save_file_name", "file_name",
    "extension", "size", "size_format", "link_url", "content_hash", "tenant_id", "created_by",
    "updated_by", "deleted_by", "created_at", "updated_at", "deleted_at",
];

// ===================== DTO =====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDto {
    pub id: i64,
    /// proto 枚举名（MINIO/ALIYUN/.../LOCAL）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_guid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    /// uint64 → protojson 字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

fn to_dto(row: &FileRow) -> FileDto {
    FileDto {
        id: row.id,
        provider: row.provider.clone(),
        bucket_name: row.bucket_name.clone(),
        file_directory: row.file_directory.clone(),
        file_guid: row.file_guid.clone(),
        save_file_name: row.save_file_name.clone(),
        file_name: row.file_name.clone(),
        extension: row.extension.clone(),
        size: row.size.map(|v| v.to_string()),
        size_format: row.size_format.clone(),
        link_url: row.link_url.clone(),
        content_hash: row.content_hash.clone(),
        tenant_id: row.tenant_id,
        tenant_name: row.tenant_name.clone(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

// ===================== 请求体 =====================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileData {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub bucket_name: Option<String>,
    #[serde(default)]
    pub file_directory: Option<String>,
    #[serde(default)]
    pub file_guid: Option<String>,
    #[serde(default)]
    pub save_file_name: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub extension: Option<String>,
    #[serde(default)]
    pub size: Option<i64>,
    #[serde(default)]
    pub size_format: Option<String>,
    #[serde(default)]
    pub link_url: Option<String>,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileBody {
    #[serde(default)]
    pub data: Option<FileData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<FileData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// provider 归一：仅接受 proto OSSProvider 枚举名（对齐 Go converter）。
fn normalize_provider(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "UNKNOWN" | "MINIO" | "ALIYUN" | "QINIU" | "TENCENT" | "AWS" | "GOOGLE" | "AZURE"
        | "BAIDU" | "HUAWEI" | "LOCAL" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid provider: {v}"))),
    }
}

fn db(state: &AppState) -> Result<sqlx::AnyPool, AppError> {
    crate::handlers::script::db_of(state)
}

// ===================== CRUD =====================

pub async fn file_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, FILE_COLUMNS)?;
    let repo = FileRepo::new(db(&state)?);

    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("f.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, FILE_COLUMNS)?;
        order_parts.push(format!("\"f\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");

    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    let dtos: Vec<FileDto> = rows.iter().map(to_dto).collect();
    Ok(json_ok(ListResponse::new(dtos, total)))
}

pub async fn file_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = FileRepo::new(db(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("file not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn file_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateFileBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let repo = FileRepo::new(db(&state)?);

    let provider = data
        .provider
        .as_deref()
        .map(normalize_provider)
        .transpose()?
        .unwrap_or_else(|| "MINIO".into());
    // size 写入时自动计算 sizeFormat（对齐 Go repo.Create）
    let size_format = data
        .size_format
        .clone()
        .or_else(|| data.size.map(storage::format_size));

    repo.create(
        &provider,
        data.bucket_name.as_deref(),
        data.file_directory.as_deref(),
        data.file_guid.as_deref(),
        data.save_file_name.as_deref(),
        data.file_name.as_deref(),
        data.extension.as_deref(),
        data.size,
        size_format.as_deref(),
        data.link_url.as_deref(),
        data.content_hash.as_deref(),
        data.tenant_id,
        operator.user_id,
    )
    .await?;
    Ok(json_empty())
}

pub async fn file_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateFileBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let repo = FileRepo::new(db(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go repo.Update allowMissing 分支）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        let provider = data
            .provider
            .as_deref()
            .map(normalize_provider)
            .transpose()?
            .unwrap_or_else(|| "MINIO".into());
        let size_format = data
            .size_format
            .clone()
            .or_else(|| data.size.map(storage::format_size));
        repo.create(
            &provider,
            data.bucket_name.as_deref(),
            data.file_directory.as_deref(),
            data.file_guid.as_deref(),
            data.save_file_name.as_deref(),
            data.file_name.as_deref(),
            data.extension.as_deref(),
            data.size,
            size_format.as_deref(),
            data.link_url.as_deref(),
            data.content_hash.as_deref(),
            data.tenant_id,
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    let provider = data.provider.as_deref().map(normalize_provider).transpose()?;
    let size_format = data
        .size_format
        .clone()
        .or_else(|| data.size.map(storage::format_size));
    let updated = repo
        .update(
            id,
            provider.as_deref(),
            data.bucket_name.as_deref(),
            data.file_directory.as_deref(),
            data.file_guid.as_deref(),
            data.save_file_name.as_deref(),
            data.file_name.as_deref(),
            data.extension.as_deref(),
            data.size,
            size_format.as_deref(),
            data.link_url.as_deref(),
            data.content_hash.as_deref(),
            data.tenant_id,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("file not found".into()));
    }
    Ok(json_empty())
}

pub async fn file_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = FileRepo::new(db(&state)?);
    // 先取元数据再删（对齐 Go FileService.Delete：先 Get 后删，删库后删对象）
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("file not found".into()))?;
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("file not found".into()));
    }
    // 同步删除本地存储对象（对齐 Go mc.DeleteFile）；失败保留告警不吞错
    if let (Some(bucket), Some(save_name)) = (row.bucket_name.as_deref(), row.save_file_name.as_deref())
    {
        let storage = storage::LocalStorage::new(state.config.upload_dir.clone());
        let object_key = storage::join_object_path(row.file_directory.as_deref().unwrap_or(""), save_name);
        if let Err(e) = storage.delete(bucket, &object_key) {
            tracing::warn!(error = %e, id, "delete storage object failed (db row already removed)");
        }
    }
    let _ = operator;
    Ok(json_empty())
}

// ===================== 本地存储访问助手 =====================

/// 从 AppState 读取本地磁盘存储（根目录 = config.upload_dir）。
pub(crate) fn local_storage(state: &AppState) -> storage::LocalStorage {
    storage::LocalStorage::new(state.config.upload_dir.clone())
}