// menu 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// List 返回扁平列表（admin 端 List treeTravel=false，children 省略）；Get 同。
// meta 为 JSON 透传（键 camelCase，对齐 protojson MenuMeta）；module 输出 proto 枚举名 MODULE_*。
// SyncMenus 对齐 Go：先 Truncate 全表，再递归插入（父先插、以父 ID 为子 parent_id、
// module 由 component 前缀推断 ComponentToModule）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::menu::{MenuRepo, MenuRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列（sys_menus 真实列名，snake_case）
const MENU_COLUMNS: &[&str] = &[
    "id", "type", "status", "path", "redirect", "alias", "name", "component", "module",
    "parent_id", "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuDto {
    pub id: i64,
    /// proto 枚举名（ON/OFF）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// proto 枚举名（CATALOG/MENU/BUTTON/EMBEDDED/LINK）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// 路由元信息（JSON 透传）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    /// proto 枚举名（MODULE_DASHBOARD 等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,
    /// 子节点树（List/Get 不返回，空则省略，对齐 Go treeTravel=false）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<MenuDto>,
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

/// 菜单行 → DTO（List/Get 返回扁平节点，children 恒为空）
pub fn to_dto(row: &MenuRow) -> MenuDto {
    MenuDto {
        id: row.id,
        status: row.status.clone(),
        r#type: row.r#type.clone(),
        path: row.path.clone(),
        redirect: row.redirect.clone(),
        alias: row.alias.clone(),
        name: row.name.clone(),
        component: row.component.clone(),
        meta: row.meta.clone(),
        module: row.module.as_deref().map(module_from_db),
        parent_id: row.parent_id,
        children: Vec::new(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuData {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub redirect: Option<String>,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub meta: Option<Value>,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub parent_id: Option<i64>,
    /// 子节点树（SyncMenus 使用；Create/Update 忽略）
    #[serde(default)]
    pub children: Option<Vec<MenuData>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMenuBody {
    #[serde(default)]
    pub data: Option<MenuData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMenuBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<MenuData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMenusBody {
    #[serde(default)]
    pub items: Option<Vec<MenuData>>,
}

/// DB 值（DASHBOARD）→ proto 枚举名（MODULE_DASHBOARD）
fn module_from_db(v: &str) -> String {
    format!("MODULE_{v}")
}

/// proto 枚举名（MODULE_DASHBOARD）→ DB 值（DASHBOARD）；MODULE_UNSPECIFIED/空 → None
fn module_to_db(v: &str) -> Result<Option<String>, AppError> {
    let up = v.to_ascii_uppercase();
    let name = up.strip_prefix("MODULE_").unwrap_or(&up);
    match name {
        "" | "UNSPECIFIED" => Ok(None),
        "DASHBOARD" | "OPM" | "SYSTEM" | "DICT" | "TENANT" | "PERMISSION" | "LOG"
        | "INTERNAL_MESSAGE" | "FILE" | "TASK" => Ok(Some(name.to_string())),
        _ => Err(AppError::Validation(format!("invalid menu module: {v}"))),
    }
}

/// type 归一：仅接受 proto 枚举名（CATALOG/MENU/BUTTON/EMBEDDED/LINK）
fn normalize_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "CATALOG" | "MENU" | "BUTTON" | "EMBEDDED" | "LINK" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid menu type: {v}"))),
    }
}

/// status 归一：仅接受 ON/OFF
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid menu status: {v}"))),
    }
}

/// 对齐 constants.ComponentToModule：由组件路径前缀推断业务模块；容器/未知 → None
fn component_to_module(component: Option<&str>) -> Option<String> {
    let c = component.unwrap_or("");
    let m = if c == "BasicLayout" || c.is_empty() {
        None
    } else if c.starts_with("dashboard/") {
        Some("DASHBOARD")
    } else if c.starts_with("app/opm/") {
        Some("OPM")
    } else if c.starts_with("app/system/") {
        Some("SYSTEM")
    } else if c.starts_with("app/dict/") {
        Some("DICT")
    } else if c.starts_with("app/tenant/") {
        Some("TENANT")
    } else if c.starts_with("app/permission/") {
        Some("PERMISSION")
    } else if c.starts_with("app/log/") {
        Some("LOG")
    } else if c.starts_with("app/internal_message/") {
        Some("INTERNAL_MESSAGE")
    } else if c.starts_with("app/file/") {
        Some("FILE")
    } else if c.starts_with("app/task/") {
        Some("TASK")
    } else {
        None
    };
    m.map(str::to_string)
}

/// 校验枚举字段（status/type/module）
fn validate_data(data: &MenuData) -> Result<(), AppError> {
    if let Some(s) = &data.status {
        normalize_status(s)?;
    }
    if let Some(t) = &data.r#type {
        normalize_type(t)?;
    }
    if let Some(m) = &data.module {
        module_to_db(m)?;
    }
    Ok(())
}

/// 创建共用逻辑（menu_create 与 update-allowMissing 转为 Create 共用）：
/// type/status 未提供时落 DB 默认（MENU/ON），parent_id=0 或缺失按无父级处理。
async fn create_menu(
    repo: &MenuRepo,
    data: &MenuData,
    operator: &Operator,
) -> Result<(), AppError> {
    validate_data(data)?;
    let r#type = data.r#type.as_deref().map(normalize_type).transpose()?;
    let status = data.status.as_deref().map(normalize_status).transpose()?;
    let module = match &data.module {
        Some(m) => module_to_db(m)?,
        None => None,
    };
    let parent_id = data.parent_id.filter(|v| *v > 0);
    repo.create(
        r#type.as_deref().or(Some("MENU")),
        data.path.as_deref(),
        data.redirect.as_deref(),
        data.alias.as_deref(),
        data.name.as_deref(),
        data.component.as_deref(),
        data.meta.as_ref(),
        module.as_deref(),
        parent_id,
        status.as_deref().or(Some("ON")),
        operator.user_id,
    )
    .await?;
    Ok(())
}

pub async fn menu_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, MENU_COLUMNS)?;
    let db = crate::handlers::script::db_of(&state)?;
    let repo = MenuRepo::new(db);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("m.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, MENU_COLUMNS)?;
        order_parts.push(format!("\"m\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    Ok(json_ok(ListResponse::new(
        rows.iter().map(to_dto).collect(),
        total,
    )))
}

pub async fn menu_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = MenuRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("menu not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn menu_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateMenuBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let repo = MenuRepo::new(crate::handlers::script::db_of(&state)?);
    create_menu(&repo, &data, &operator).await?;
    Ok(json_empty())
}

pub async fn menu_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateMenuBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let repo = MenuRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        create_menu(&repo, &data, &operator).await?;
        return Ok(json_empty());
    }

    validate_data(&data)?;
    let r#type = data.r#type.as_deref().map(normalize_type).transpose()?;
    let status = data.status.as_deref().map(normalize_status).transpose()?;
    let module = match &data.module {
        Some(m) => module_to_db(m)?,
        None => None,
    };
    // parent_id=0 或缺失：不改动父级（对齐 Go parent_id=0 按无父级处理）
    let parent_id = data.parent_id.filter(|v| *v > 0);

    let updated = repo
        .update(
            id,
            r#type.as_deref(),
            data.path.as_deref(),
            data.redirect.as_deref(),
            data.alias.as_deref(),
            data.name.as_deref(),
            data.component.as_deref(),
            data.meta.as_ref(),
            module.as_deref(),
            parent_id,
            status.as_deref(),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("menu not found".into()));
    }
    Ok(json_empty())
}

pub async fn menu_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = MenuRepo::new(crate::handlers::script::db_of(&state)?);
    // 递归删除该节点及全部后代（对齐 Go QueryAllChildrenIds）
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("menu not found".into()));
    }
    Ok(json_empty())
}

/// 递归插入菜单树：先插入父节点拿到数据库 ID，再以该 ID 为 parent_id 递归子节点
/// （对齐 Go syncMenuTree：清空前端 ID 由库自增、module 由 component 推断覆盖）。
fn sync_tree<'a>(
    repo: &'a MenuRepo,
    items: &'a [MenuData],
    parent_id: Option<i64>,
    operator_id: i64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64, AppError>> + Send + 'a>> {
    Box::pin(async move {
        let mut count: u64 = 0;
        for m in items {
            validate_data(m)?;
            let r#type = m.r#type.as_deref().map(normalize_type).transpose()?;
            let status = m.status.as_deref().map(normalize_status).transpose()?;
            // 对齐 Go：module 一律由 component 推断（ComponentToModule），客户端传值被覆盖
            let module = component_to_module(m.component.as_deref());
            let children = m.children.clone().unwrap_or_default();
            let new_id = repo
                .create(
                    r#type.as_deref().or(Some("MENU")),
                    m.path.as_deref(),
                    m.redirect.as_deref(),
                    m.alias.as_deref(),
                    m.name.as_deref(),
                    m.component.as_deref(),
                    m.meta.as_ref(),
                    module.as_deref(),
                    parent_id,
                    status.as_deref().or(Some("ON")),
                    operator_id,
                )
                .await?;
            count += 1;
            if !children.is_empty() {
                count += sync_tree(repo, &children, Some(new_id), operator_id).await?;
            }
        }
        Ok(count)
    })
}

/// POST /menus/sync：清空现有菜单数据后递归插入前端传入的树形菜单（对齐 Go SyncMenus）。
pub async fn menu_sync_menus(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<SyncMenusBody>,
) -> Result<impl IntoResponse, AppError> {
    let items = body.items.unwrap_or_default();
    let repo = MenuRepo::new(crate::handlers::script::db_of(&state)?);
    repo.truncate().await?;
    sync_tree(&repo, &items, None, operator.user_id).await?;
    Ok(json_empty())
}
