// permission 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略）。
// 对齐 Go permission_service.go：
//  - List/Get 回填 groupName + menuIds + apiIds；
//  - Create/Update/Delete 后重置权限策略（Rust 无 authorizer，ResetPolicies 为 no-op）；
//  - SyncPermissions 全量重建业务权限/分组（保留 sys:* 系统权限），menu→code/api→code 转换对齐 converter。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::{HashMap, HashSet};

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::permission::{PermissionNew, PermissionRepo, PermissionRow};
use crate::repos::permission_group::PermissionGroupRepo;
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const PERMISSION_COLUMNS: &[&str] = &[
    "id", "name", "code", "group_id", "status", "description", "created_by", "updated_by",
    "created_at", "updated_at",
];

// ===================== DTO =====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// ON | OFF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    // protojson 对 repeated 字段始终输出（空时为 []），不做 skip，保持与 Go wire 一致。
    pub menu_ids: Vec<i64>,
    pub api_ids: Vec<i64>,
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

fn to_dto(
    row: &PermissionRow,
    menu_ids: &HashMap<i64, Vec<i64>>,
    api_ids: &HashMap<i64, Vec<i64>>,
) -> PermissionDto {
    PermissionDto {
        id: row.id,
        name: row.name.clone(),
        code: row.code.clone(),
        description: row.description.clone(),
        status: row.status.clone(),
        group_id: row.group_id,
        group_name: row.group_name.clone(),
        menu_ids: menu_ids.get(&row.id).cloned().unwrap_or_default(),
        api_ids: api_ids.get(&row.id).cloned().unwrap_or_default(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

async fn enrich(
    repo: &PermissionRepo,
    rows: &[PermissionRow],
) -> Result<(HashMap<i64, Vec<i64>>, HashMap<i64, Vec<i64>>), AppError> {
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let menu_map = repo.list_menu_ids(&ids).await?;
    let api_map = repo.list_api_ids(&ids).await?;
    Ok((menu_map, api_map))
}

// ===================== 请求体 =====================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub group_id: Option<i64>,
    #[serde(default)]
    pub menu_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub api_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePermissionBody {
    #[serde(default)]
    pub data: Option<PermissionData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePermissionBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<PermissionData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid permission status: {v}"))),
    }
}

fn validate_data(data: &PermissionData) -> Result<(), AppError> {
    if data.name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("permission name is required".into()));
    }
    if data.code.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("permission code is required".into()));
    }
    if let Some(s) = &data.status {
        normalize_status(s)?;
    }
    Ok(())
}

fn db(state: &AppState) -> Result<sqlx::AnyPool, AppError> {
    crate::handlers::script::db_of(state)
}

// ===================== CRUD =====================

pub async fn permission_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, PERMISSION_COLUMNS)?;
    let repo = PermissionRepo::new(db(&state)?);

    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("p.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, PERMISSION_COLUMNS)?;
        order_parts.push(format!("\"p\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");

    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    let (menu_map, api_map) = enrich(&repo, &rows).await?;
    let dtos: Vec<PermissionDto> = rows
        .iter()
        .map(|r| to_dto(r, &menu_map, &api_map))
        .collect();
    Ok(json_ok(ListResponse::new(dtos, total)))
}

pub async fn permission_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = PermissionRepo::new(db(&state)?);
    // 支持 code 查询（oneof id/code 语义；未提供时以路径 id 为准）
    let row = match params.get("code") {
        Some(code) if !code.trim().is_empty() => repo
            .get_by_code(code)
            .await?
            .ok_or_else(|| AppError::NotFound("permission not found".into()))?,
        _ => repo
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound("permission not found".into()))?,
    };
    let ids = vec![row.id];
    let menu_map = repo.list_menu_ids(&ids).await?;
    let api_map = repo.list_api_ids(&ids).await?;
    Ok(json_ok(to_dto(&row, &menu_map, &api_map)))
}

pub async fn permission_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreatePermissionBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    validate_data(&data)?;
    let code = data.code.clone().unwrap_or_default().trim().to_string();

    let repo = PermissionRepo::new(db(&state)?);
    if repo.exists_code(&code, 0).await? {
        return Err(AppError::Conflict("permission code already exists".into()));
    }

    let status = data
        .status
        .as_deref()
        .map(normalize_status)
        .transpose()?
        .unwrap_or_else(|| "ON".into());
    repo.create(
        data.name.as_deref(),
        Some(&code),
        data.group_id,
        Some(&status),
        data.description.as_deref(),
        data.menu_ids.as_deref().unwrap_or(&[]),
        data.api_ids.as_deref().unwrap_or(&[]),
        operator.user_id,
    )
    .await?;
    Ok(json_empty())
}

pub async fn permission_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdatePermissionBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let repo = PermissionRepo::new(db(&state)?);

    let code = data.code.as_deref().map(str::trim).unwrap_or("");
    if !code.is_empty() && repo.exists_code(code, id).await? {
        return Err(AppError::Conflict("permission code already exists".into()));
    }

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go updateMask 忽略）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let status = data
            .status
            .as_deref()
            .map(normalize_status)
            .transpose()?
            .unwrap_or_else(|| "ON".into());
        repo.create(
            data.name.as_deref(),
            Some(code),
            data.group_id,
            Some(&status),
            data.description.as_deref(),
            data.menu_ids.as_deref().unwrap_or(&[]),
            data.api_ids.as_deref().unwrap_or(&[]),
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    if data.name.is_some() || data.code.is_some() {
        validate_data(&data)?;
    }
    let status = data.status.as_deref().map(normalize_status).transpose()?;

    let name = data.name.as_deref().map(str::trim).filter(|v| !v.is_empty());
    let updated = repo
        .update(
            id,
            name,
            (if code.is_empty() { None } else { Some(code) }),
            data.group_id,
            status.as_deref(),
            data.description.as_deref(),
            data.menu_ids.as_deref(),
            data.api_ids.as_deref(),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("permission not found".into()));
    }
    Ok(json_empty())
}

pub async fn permission_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = PermissionRepo::new(db(&state)?);
    // 支持 code / groupId 删除（oneof id/code/groupId 语义）
    if let Some(code) = params.get("code").filter(|v| !v.trim().is_empty()) {
        let row = repo
            .get_by_code(code)
            .await?
            .ok_or_else(|| AppError::NotFound("permission not found".into()))?;
        let _ = repo.delete(row.id).await?;
        return Ok(json_empty());
    }
    if let Some(group_id) = params.get("groupId").and_then(|v| v.parse::<i64>().ok()) {
        // groupId 删除语义：清理该组下全部权限点
        let sql = format!("delete from sys_permissions where group_id = {group_id}");
        let _ = sqlx::query::<sqlx::Any>(&sql).execute(&repo.db).await;
        return Ok(json_empty());
    }
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("permission not found".into()));
    }
    Ok(json_empty())
}

// ===================== SyncPermissions =====================

/// 菜单类型 → action 后缀（对齐 converter.MenuPermissionConverter.typeToAction）
fn menu_type_to_action(title: &str, mtype: &str) -> String {
    match mtype {
        "CATALOG" => "dir".into(),
        "MENU" => "view".into(),
        "EMBEDDED" => "view".into(),
        "LINK" => "jump".into(),
        "BUTTON" => button_action(title),
        _ => String::new(),
    }
}

/// 按钮动作（对齐 converter.buttonAction 关键词表）
fn button_action(title: &str) -> String {
    let title = title.trim().to_lowercase();
    let add = ["add", "addto", "add+", "create", "new", "plus", "append", "新增", "添加", "创建"];
    let edit = ["edit", "update", "modify", "save", "patch", "保存", "修改", "更新", "编辑"];
    let delete = ["delete", "del", "remove", "destroy", "drop", "discard", "trash", "删除", "移除", "弃用", "清除"];
    let import_keys = ["import", "importcsv", "importexcel", "导入", "导入为"];
    let export_keys = ["export", "download", "exportcsv", "exportexcel", "导出", "下载", "导出为"];

    let has = |keys: &[&str]| {
        keys.iter().any(|k| {
            title.split(|c: char| !c.is_alphanumeric() && !c.is_ascii()).any(|t| {
                let t = t.trim();
                !t.is_empty() && (t == *k || t.starts_with(k))
            }) || title.contains(k)
        })
    };

    if has(&add) {
        return "create".into();
    }
    if has(&edit) {
        return "edit".into();
    }
    if has(&delete) {
        return "delete".into();
    }
    if has(&import_keys) {
        return "import".into();
    }
    if has(&export_keys) {
        return "export".into();
    }
    "act".into()
}

/// 简单英文单词单数化（对齐 jinzhu/inflection 的常用规则子集）
fn singularize(s: &str) -> String {
    if s.is_empty() {
        return s.into();
    }
    if s.ends_with("ies") && s.len() > 3 {
        return format!("{}y", &s[..s.len() - 3]);
    }
    if s.ends_with("ses") || s.ends_with("xes") || s.ends_with("zes") || s.ends_with("ches") || s.ends_with("shes") {
        return s[..s.len() - 2].to_string();
    }
    if s.ends_with('s') && !s.ends_with("ss") && s.len() > 1 {
        return s[..s.len() - 1].to_string();
    }
    s.into()
}

/// 菜单 code 转换（对齐 converter.ConvertCode）
fn convert_menu_code(full_path: &str, title: &str, mtype: &str) -> String {
    let path = full_path.trim().trim_matches('/');
    if path.is_empty() {
        return String::new();
    }
    let mut paths: Vec<String> = path
        .split('/')
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.trim().to_string())
        .collect();
    if paths.is_empty() {
        return String::new();
    }
    // 去掉第一个路径段（对齐 paths[1:]），首段为根目录路径
    if paths.len() > 1 {
        paths.drain(..1);
    }
    let mut segs: Vec<String> = Vec::new();
    for p in paths {
        if p.starts_with(':') {
            continue;
        }
        let s = singularize(&p);
        if !s.trim().is_empty() {
            segs.push(s);
        }
    }
    let perm_base = segs.join(":");
    if perm_base.is_empty() {
        return String::new();
    }
    let action = menu_type_to_action(title, mtype);
    if action.is_empty() {
        perm_base
    } else {
        format!("{perm_base}:{action}")
    }
}

/// 计算每个菜单的完整路径（对齐 ComposeMenuPaths：memoization + 循环检测）
fn compose_menu_paths(menus: &[(i64, String, String, i64)]) -> HashMap<i64, String> {
    let map: HashMap<i64, (String, i64)> = menus
        .iter()
        .map(|(id, path, _, parent)| (*id, (path.clone(), *parent)))
        .collect();
    let mut memo: HashMap<i64, String> = HashMap::new();

    fn compute(
        id: i64,
        map: &HashMap<i64, (String, i64)>,
        memo: &mut HashMap<i64, String>,
        seen: &mut HashSet<i64>,
    ) -> String {
        if let Some(v) = memo.get(&id) {
            return v.clone();
        }
        if seen.contains(&id) {
            let v = map.get(&id).map(|(p, _)| p.trim_matches('/').to_string()).unwrap_or_default();
            memo.insert(id, v.clone());
            return v;
        }
        let Some((part, parent_id)) = map.get(&id) else {
            memo.insert(id, String::new());
            return String::new();
        };
        seen.insert(id);
        let parent_full = if *parent_id == 0 || *parent_id == id {
            String::new()
        } else {
            compute(*parent_id, map, memo, seen)
        };
        seen.remove(&id);

        let part = part.trim_matches('/').to_string();
        let v = match (parent_full.as_str(), part.as_str()) {
            ("", "") => String::new(),
            ("", p) => p.to_string(),
            (pf, "") => pf.to_string(),
            (pf, p) => format!("{pf}/{p}"),
        };
        memo.insert(id, v.clone());
        v
    }

    for (id, _, _, _) in menus {
        compute(*id, &map, &mut memo, &mut HashSet::new());
    }
    memo
}

/// API path → resource（对齐 converter.pathToResource：去版本前缀 + 去参数段 + 首段单数）
fn api_path_to_resource(path: &str) -> String {
    let p = path.trim().trim_matches('/');
    if p.is_empty() {
        return String::new();
    }
    let mut parts: Vec<&str> = p.split('/').collect();
    // 去开头 api
    if parts.first().is_some_and(|s| s.eq_ignore_ascii_case("api")) {
        parts.remove(0);
    }
    // 去版本段（v1/v2 -> 所在索引 <= 1 时移除到该段）
    let mut idx = -1i32;
    for (i, seg) in parts.iter().enumerate() {
        let lower = seg.to_lowercase();
        let is_version = lower.starts_with('v')
            && lower[1..].chars().next().unwrap_or('\0').is_ascii_digit()
            && lower[1..].chars().all(|c| c.is_ascii_digit() || c == '.');
        if is_version {
            idx = i as i32;
            break;
        }
    }
    if idx >= 0 && idx <= 1 {
        let cut = (idx + 1) as usize;
        if cut <= parts.len() {
            parts.drain(..cut);
        } else {
            parts.clear();
        }
    }
    parts.retain(|s| {
        let s = s.trim();
        !s.is_empty() && !s.eq_ignore_ascii_case("api")
    });
    if parts.is_empty() {
        return String::new();
    }
    // 去除参数段 {id}
    let clean: Vec<&str> = parts
        .iter()
        .copied()
        .filter(|s| !(s.trim().starts_with('{') && s.trim().ends_with('}')))
        .collect();
    let Some(first) = clean.first().copied() else {
        return String::new();
    };
    // 冒号段取第一段 + 单数化
    let raw = first.trim().trim_matches(':');
    if raw.is_empty() {
        return String::new();
    }
    let mut resource = singularize(raw);
    if resource.is_empty() {
        return String::new();
    }
    if raw.contains(':') {
        let first_part = raw.split(':').next().unwrap_or(raw);
        resource = singularize(first_part);
    }
    resource
}

/// API method+path → code（对齐 converter.ConvertCodeByPath）
fn convert_api_code(method: &str, path: &str) -> String {
    let resource = api_path_to_resource(path);
    if resource.is_empty() {
        return String::new();
    }
    let action = if path.ends_with("/list") {
        "view".to_string()
    } else {
        match method.to_ascii_uppercase().as_str() {
            "GET" => "view",
            "POST" => "create",
            "PUT" | "PATCH" => "edit",
            "DELETE" => "delete",
            _ => return String::new(),
        }
        .to_string()
    };
    format!("{resource}:{action}")
}

/// PascalCase（对齐 stringcase.ToPascalCase 的常见子集：分隔符 → 词首大写）
fn pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for c in s.chars() {
        if c == '_' || c == '-' || c == ':' || c == ' ' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// 从菜单路径提取模块名（对齐 menuPathToModuleName）
fn menu_path_to_module(menu_path: &str) -> String {
    let parts: Vec<&str> = menu_path.split('/').collect();
    if parts.len() > 1 {
        let module = parts[1].trim().to_string();
        if !module.is_empty() {
            return module;
        }
    }
    "biz".into()
}

#[derive(Clone)]
struct SyncMenu {
    id: i64,
    name: String,
    path: String,
    parent_id: i64,
    mtype: String,
}

struct SyncApi {
    id: i64,
    method: String,
    path: String,
}

/// 列出启用的菜单（对齐 Go List status=ON + orderBy=id desc）
async fn list_enabled_menus(db: &sqlx::AnyPool) -> Result<Vec<SyncMenu>, AppError> {
    let sql = "select m.id, m.name, m.path, m.parent_id, m.type from sys_menus m \
               where m.status = 'ON' and m.deleted_at is null order by m.id desc";
    let rows = sqlx::query::<sqlx::Any>(sql)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Internal {
            context: "list enabled menus failed".into(),
            source: Some(Box::new(e)),
        })?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id = row.try_get::<i64, _>("id").map_err(|e| AppError::Internal {
            context: "menu id decode failed".into(),
            source: Some(Box::new(e)),
        })?;
        let name = row.try_get::<Option<String>, _>("name").ok().flatten().unwrap_or_default();
        let path = row.try_get::<Option<String>, _>("path").ok().flatten().unwrap_or_default();
        let parent_id = row.try_get::<i64, _>("parent_id").unwrap_or(0);
        let mtype = row.try_get::<Option<String>, _>("type").ok().flatten().unwrap_or_else(|| "MENU".into());
        out.push(SyncMenu { id, name, path, parent_id, mtype });
    }
    Ok(out)
}

/// 列出启用状态下的全部 API 资源（对齐 Go appendAPis 的 status=ON 查询）
async fn list_enabled_apis(db: &sqlx::AnyPool) -> Result<Vec<SyncApi>, AppError> {
    let sql = "select a.id, a.method, a.path from sys_apis a \
               where a.status = 'ON' and a.deleted_at is null order by a.module, a.path desc, a.operation desc";
    let rows = sqlx::query::<sqlx::Any>(sql)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Internal {
            context: "list enabled apis failed".into(),
            source: Some(Box::new(e)),
        })?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id = row.try_get::<i64, _>("id").map_err(|e| AppError::Internal {
            context: "api id decode failed".into(),
            source: Some(Box::new(e)),
        })?;
        let method = row.try_get::<Option<String>, _>("method").ok().flatten().unwrap_or_default();
        let path = row.try_get::<Option<String>, _>("path").ok().flatten().unwrap_or_default();
        out.push(SyncApi { id, method, path });
    }
    Ok(out)
}

/// sync:perms 过程中内存态权限（含所属 module，用于回填分组）
struct PermAcc {
    name: String,
    code: String,
    module: String,
    menu_ids: Vec<i64>,
    api_ids: Vec<i64>,
    group_id: Option<i64>,
}

pub async fn permission_sync_permissions(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db(&state)?;

    // 1. 清理业务权限/分组（保留 sys:* 与 module='sys'）
    let repo = PermissionRepo::new(db.clone());
    repo.truncate_biz_permissions().await?;
    let group_repo = PermissionGroupRepo::new(db.clone());
    group_repo.truncate_biz_groups().await?;

    // 2. 查询启用菜单并计算完整路径
    let menus = list_enabled_menus(&db).await?;
    let path_map = compose_menu_paths(
        &menus
            .iter()
            .map(|m| (m.id, m.path.clone(), m.name.clone(), m.parent_id))
            .collect::<Vec<_>>(),
    );

    let mut sorted = menus.clone();
    sorted.sort_by_key(|m| m.parent_id);

    // 分组：(name, module)；首位固定「未分类」
    let mut permission_groups: Vec<(String, String)> = Vec::new();
    permission_groups.push(("未分类".into(), "uncategorized".into()));

    let mut permissions: Vec<PermAcc> = Vec::new();
    let mut map_permissions: HashMap<String, Vec<usize>> = HashMap::new(); // module -> perm indexes

    for menu in &sorted {
        let full_path = path_map.get(&menu.id).cloned().unwrap_or_default();
        let code = convert_menu_code(&full_path, &menu.name, &menu.mtype);
        if code.is_empty() {
            continue;
        }
        let module = menu_path_to_module(&full_path);

        if menu.mtype == "CATALOG" {
            permission_groups.push((menu.name.clone(), module.clone()));
        }

        let idx = permissions.len();
        permissions.push(PermAcc {
            name: menu.name.clone(),
            code,
            module: module.clone(),
            menu_ids: vec![menu.id],
            api_ids: Vec::new(),
            group_id: None,
        });
        map_permissions.entry(module).or_default().push(idx);
    }

    // 3. 为权限追加对应 API 关联（对齐 appendAPis：code → apiIds 映射）
    let apis = list_enabled_apis(&db).await?;
    // code -> apis；module 依据已有权限 code 前缀匹配
    let mut codes: HashMap<String, Vec<i64>> = HashMap::new();
    for api in &apis {
        let code = convert_api_code(&api.method, &api.path);
        if code.is_empty() {
            continue;
        }
        codes.entry(code).or_default().push(api.id);
    }

    for perm in &mut permissions {
        if let Some(apis) = codes.remove(&perm.code) {
            perm.api_ids = apis;
        }
    }
    // 未匹配的 code 新建权限（对齐 appendAPis 末尾段，归入模块前缀最近的分组）
    for (code, apis) in codes {
        let module = permissions
            .iter()
            .find(|p| code.split(':').next().unwrap_or("").starts_with(p.code.split(':').next().unwrap_or("")))
            .map(|p| p.module.clone())
            .unwrap_or_else(|| "uncategorized".into());
        let idx = permissions.len();
        permissions.push(PermAcc {
            name: pascal_case(&code.replace(':', "_")),
            code: code.clone(),
            module: module.clone(),
            menu_ids: Vec::new(),
            api_ids: apis,
            group_id: None,
        });
        map_permissions.entry(module).or_default().push(idx);
    }

    // 4. 批量创建权限组并拿回 id，按 module 回填权限 group_id
    let mut module_group: HashMap<String, i64> = HashMap::new();
    for (i, (name, module)) in permission_groups.iter().enumerate() {
        let gid = group_repo
            .create(
                name,
                Some(module),
                Some(i as i64 + 1),
                Some("ON"),
                None,
                None,
                operator.user_id,
            )
            .await?;
        module_group.insert(module.clone(), gid);
    }
    for (module, idxs) in &map_permissions {
        if let Some(gid) = module_group.get(module) {
            for idx in idxs {
                if let Some(p) = permissions.get_mut(*idx) {
                    p.group_id = Some(*gid);
                }
            }
        }
    }

    // 5. 批量创建权限
    let batch: Vec<PermissionNew> = permissions
        .iter()
        .map(|p| PermissionNew {
            name: Some(p.name.clone()),
            code: Some(p.code.clone()),
            group_id: p.group_id,
            menu_ids: p.menu_ids.clone(),
            api_ids: p.api_ids.clone(),
        })
        .collect();
    repo.batch_create(&batch, operator.user_id).await?;

    // 6. 重置权限策略（Rust 无 casbin authorizer，no-op）
    let _ = (&state, batch);
    Ok(json_empty())
}