// admin_portal 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、repeated 空值省略）。
// 数据流对齐 Go admin_portal_service.go + 各 repo：
//   user(role_ids) → sys_user_roles → sys_role_permissions(permission_ids)
//   → sys_permissions(codes) / sys_permission_menus(menu_ids) → sys_menus 树
// routes 额外按租户套餐白名单过滤 module（仅 tenant_id>0，平台管理员不过滤）。

use axum::extract::State;
use axum::response::IntoResponse;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use sqlx::Row;

use crate::error::AppError;
use crate::handlers::script::db_of;
use crate::middleware::Operator;
use crate::response::json_ok;
use crate::state::AppState;

/// 路由项（对齐 permission.service.v1.MenuRouteItem：NULL 字段省略、children 为空省略）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuRouteItemDto {
    // protojson 对 repeated 子消息恒输出（叶子为 []），不做 skip，保持与 Go wire 一致。
    pub children: Vec<MenuRouteItemDto>,
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
    /// meta 为 jsonb 透传（DB 已按 protojson camelCase 键存储）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRouteResponseDto {
    /// repeated items：无菜单时省略（对齐 Go fillRouteItem 返回 nil）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<MenuRouteItemDto>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPermissionCodeResponseDto {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub codes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialContextResponseDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menus: Option<Vec<MenuRouteItemDto>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
}

/// 菜单行（仅含 MenuRouteItem 需要的字段 + 树构建/白名单过滤所需的字段）
struct MenuNode {
    id: i64,
    parent_id: i64,
    path: Option<String>,
    redirect: Option<String>,
    alias: Option<String>,
    name: Option<String>,
    component: Option<String>,
    meta: Option<serde_json::Value>,
    module: Option<String>,
}

/// `$1,$2,...` 占位符（postgres 方言）。
fn in_placeholders(n: usize) -> String {
    (1..=n).map(|i| format!("${i}")).collect::<Vec<_>>().join(", ")
}

/// 用户必须存在（对齐 Go userRepo.Get 失败 → "query user failed"）。
async fn ensure_user_exists(db: &sqlx::AnyPool, user_id: i64) -> Result<(), AppError> {
    let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(
        "select 1 from sys_users where id = $1 and deleted_at is null limit 1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Internal {
        context: "query user failed".into(),
        source: Some(Box::new(e)),
    })?;
    if row.is_none() {
        return Err(AppError::Internal {
            context: "query user failed".into(),
            source: None,
        });
    }
    Ok(())
}

async fn role_ids_of_user(db: &sqlx::AnyPool, user_id: i64) -> Result<Vec<i64>, AppError> {
    let sql = "select role_id from sys_user_roles where user_id = $1 and deleted_at is null";
    let rows = sqlx::query::<sqlx::Any>(sql)
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Internal {
            context: "query user role ids failed".into(),
            source: Some(Box::new(e)),
        })?;
    let mut ids = Vec::with_capacity(rows.len());
    for row in &rows {
        if let Some(id) = row.try_get::<Option<i64>, _>("role_id").ok().flatten() {
            ids.push(id);
        }
    }
    Ok(ids)
}

async fn permission_ids_of_roles(db: &sqlx::AnyPool, role_ids: &[i64]) -> Result<Vec<i64>, AppError> {
    if role_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "select permission_id from sys_role_permissions \
         where role_id in ({}) and deleted_at is null",
        in_placeholders(role_ids.len())
    );
    let mut q = sqlx::query::<sqlx::Any>(&sql);
    for id in role_ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(db).await.map_err(|e| AppError::Internal {
        context: "query role permission ids failed".into(),
        source: Some(Box::new(e)),
    })?;
    let mut ids = Vec::with_capacity(rows.len());
    for row in &rows {
        if let Some(id) = row.try_get::<Option<i64>, _>("permission_id").ok().flatten() {
            ids.push(id);
        }
    }
    Ok(ids)
}

async fn permission_codes_of_ids(db: &sqlx::AnyPool, permission_ids: &[i64]) -> Result<Vec<String>, AppError> {
    if permission_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "select code from sys_permissions \
         where id in ({}) and deleted_at is null",
        in_placeholders(permission_ids.len())
    );
    let mut q = sqlx::query::<sqlx::Any>(&sql);
    for id in permission_ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(db).await.map_err(|e| AppError::Internal {
        context: "query permission codes failed".into(),
        source: Some(Box::new(e)),
    })?;
    let mut codes = Vec::with_capacity(rows.len());
    for row in &rows {
        if let Some(c) = row.try_get::<Option<String>, _>("code").ok().flatten() {
            codes.push(c);
        }
    }
    Ok(codes)
}

async fn menu_ids_of_permissions(db: &sqlx::AnyPool, permission_ids: &[i64]) -> Result<Vec<i64>, AppError> {
    if permission_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "select menu_id from sys_permission_menus \
         where permission_id in ({}) and deleted_at is null",
        in_placeholders(permission_ids.len())
    );
    let mut q = sqlx::query::<sqlx::Any>(&sql);
    for id in permission_ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(db).await.map_err(|e| AppError::Internal {
        context: "query permission menu ids failed".into(),
        source: Some(Box::new(e)),
    })?;
    let mut ids = Vec::with_capacity(rows.len());
    for row in &rows {
        if let Some(id) = row.try_get::<Option<i64>, _>("menu_id").ok().flatten() {
            ids.push(id);
        }
    }
    Ok(ids)
}

async fn load_menus(db: &sqlx::AnyPool, menu_ids: &[i64]) -> Result<Vec<MenuNode>, AppError> {
    if menu_ids.is_empty() {
        return Ok(Vec::new());
    }
    // 对齐 Go menuListToQueryString：id__in + type__not=BUTTON + status=ON
    let sql = format!(
        "select id, parent_id, path, redirect, alias, name, component, \
                meta::text as meta, module \
         from sys_menus \
         where id in ({}) and status = 'ON' and type <> 'BUTTON' and deleted_at is null \
         order by id",
        in_placeholders(menu_ids.len())
    );
    let mut q = sqlx::query::<sqlx::Any>(&sql);
    for id in menu_ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(db).await.map_err(|e| AppError::Internal {
        context: "list route menus failed".into(),
        source: Some(Box::new(e)),
    })?;
    let mut nodes = Vec::with_capacity(rows.len());
    for row in &rows {
        let id: i64 = row.try_get("id").map_err(|e| AppError::Internal {
            context: "menu row id decode failed".into(),
            source: Some(Box::new(e)),
        })?;
        // 种子数据的根菜单 parent_id 为 NULL（Ent 可空列），NULL 视作 0（根节点）。
        let parent_id: i64 = row
            .try_get::<Option<i64>, _>("parent_id")
            .ok()
            .flatten()
            .unwrap_or(0);
        nodes.push(MenuNode {
            id,
            parent_id,
            path: row.try_get::<Option<String>, _>("path").ok().flatten(),
            redirect: row.try_get::<Option<String>, _>("redirect").ok().flatten(),
            alias: row.try_get::<Option<String>, _>("alias").ok().flatten(),
            name: row.try_get::<Option<String>, _>("name").ok().flatten(),
            component: row.try_get::<Option<String>, _>("component").ok().flatten(),
            meta: row
                .try_get::<Option<String>, _>("meta")
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str(&s).ok()),
            module: row.try_get::<Option<String>, _>("module").ok().flatten(),
        });
    }
    Ok(nodes)
}

/// 按租户订阅套餐的模块白名单过滤根节点（对齐 Go filterMenusByPlanWhitelist）：
/// 无套餐 / 查询失败 / 白名单为空 → 清空菜单；module 为空的 catalog 容器保留。
async fn filter_roots_by_plan(
    db: &sqlx::AnyPool,
    tenant_id: i64,
    nodes: &[MenuNode],
    root_idx: &mut Vec<usize>,
) -> Result<(), AppError> {
    let row = sqlx::query::<sqlx::Any>("select plan_id from sys_tenants where id = $1 and deleted_at is null")
        .bind(tenant_id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Internal {
            context: "query tenant plan failed".into(),
            source: Some(Box::new(e)),
        })?;
    let plan_id: Option<i64> = match row {
        Some(r) => r.try_get::<Option<i64>, _>("plan_id").ok().flatten(),
        None => None,
    };
    let Some(plan_id) = plan_id else {
        root_idx.clear();
        return Ok(());
    };
    if plan_id == 0 {
        root_idx.clear();
        return Ok(());
    }

    let rows = sqlx::query::<sqlx::Any>("select module from sys_plan_modules where plan_id = $1 and deleted_at is null")
        .bind(plan_id)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Internal {
            context: "query plan modules failed".into(),
            source: Some(Box::new(e)),
        })?;
    let mut allowed: HashSet<String> = HashSet::new();
    for row in &rows {
        if let Some(m) = row.try_get::<Option<String>, _>("module").ok().flatten() {
            allowed.insert(m);
        }
    }
    if allowed.is_empty() {
        root_idx.clear();
        return Ok(());
    }

    root_idx.retain(|&i| match &nodes[i].module {
        None => true,
        Some(m) => allowed.contains(m),
    });
    Ok(())
}

/// 由 parent_id 构建树并递归填充 MenuRouteItem（对齐 Go BuildTree + fillRouteItem）：
/// parent_id=0 为根；父节点不在集合内的孤儿被跳过；children 为空时省略。
fn build_tree(nodes: &[MenuNode], root_idx: &[usize]) -> Vec<MenuRouteItemDto> {
    let mut by_parent: HashMap<i64, Vec<usize>> = HashMap::new();
    for (i, n) in nodes.iter().enumerate() {
        by_parent.entry(n.parent_id).or_default().push(i);
    }

    fn fill(
        by_parent: &HashMap<i64, Vec<usize>>,
        nodes: &[MenuNode],
        list: &[usize],
    ) -> Vec<MenuRouteItemDto> {
        let mut out = Vec::with_capacity(list.len());
        for &i in list {
            let n = &nodes[i];
            let children = by_parent
                .get(&n.id)
                .map(|sub| fill(by_parent, nodes, sub))
                .unwrap_or_default();
            out.push(MenuRouteItemDto {
                children,
                path: n.path.clone(),
                redirect: n.redirect.clone(),
                alias: n.alias.clone(),
                name: n.name.clone(),
                component: n.component.clone(),
                meta: crate::handlers::menu::normalize_meta(&n.meta),
            });
        }
        out
    }

    fill(&by_parent, nodes, root_idx)
}

/// 我的权限码列表（对齐 Go GetMyPermissionCode）。
async fn permission_codes_for_user(db: &sqlx::AnyPool, user_id: i64) -> Result<Vec<String>, AppError> {
    ensure_user_exists(db, user_id).await?;
    let role_ids = role_ids_of_user(db, user_id).await?;
    let permission_ids = permission_ids_of_roles(db, &role_ids).await?;
    permission_codes_of_ids(db, &permission_ids).await
}

/// 我的导航路由树（对齐 Go GetNavigation，含租户套餐白名单过滤）。
async fn menu_tree_for_user(
    db: &sqlx::AnyPool,
    user_id: i64,
    tenant_id: i64,
) -> Result<Option<Vec<MenuRouteItemDto>>, AppError> {
    ensure_user_exists(db, user_id).await?;
    let role_ids = role_ids_of_user(db, user_id).await?;
    let permission_ids = permission_ids_of_roles(db, &role_ids).await?;
    let menu_ids = menu_ids_of_permissions(db, &permission_ids).await?;
    let nodes = load_menus(db, &menu_ids).await?;

    let mut root_idx: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.parent_id == 0)
        .map(|(i, _)| i)
        .collect();
    if tenant_id > 0 {
        filter_roots_by_plan(db, tenant_id, &nodes, &mut root_idx).await?;
    }

    let items = build_tree(&nodes, &root_idx);
    if items.is_empty() {
        Ok(None)
    } else {
        Ok(Some(items))
    }
}

pub async fn admin_portal_get_my_permission_code(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;
    let codes = permission_codes_for_user(&db, operator.user_id).await?;
    Ok(json_ok(ListPermissionCodeResponseDto { codes }))
}

pub async fn admin_portal_get_navigation(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;
    let items = menu_tree_for_user(&db, operator.user_id, operator.tenant_id).await?;
    Ok(json_ok(ListRouteResponseDto { items }))
}

pub async fn admin_portal_get_initial_context(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;
    let menus = menu_tree_for_user(&db, operator.user_id, operator.tenant_id).await?;
    let permissions = permission_codes_for_user(&db, operator.user_id).await?;
    Ok(json_ok(InitialContextResponseDto { menus, permissions }))
}
