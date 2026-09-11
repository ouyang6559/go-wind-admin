// dict_entry 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、i18n 为 map）。
// 端点：GET/POST/DELETE /dict/entries、GET /dict/entries/by-type-code、PUT /dict/entries/{id}。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::dict_entry::{DictEntryRepo, DictEntryRow, I18nInput};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const DICT_ENTRY_COLUMNS: &[&str] = &[
    "id", "type_id", "entry_value", "numeric_value", "is_enabled", "sort_order", "tenant_id",
    "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictEntryDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub i18n: HashMap<String, DictEntryI18nDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictEntryI18nDto {
    /// proto 非 optional string：对齐 protojson 恒输出（空值为 ""）
    pub entry_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_name: Option<String>,
}

pub fn to_dto(row: &DictEntryRow) -> DictEntryDto {
    DictEntryDto {
        id: row.id,
        type_id: row.type_id,
        entry_value: row.entry_value.clone(),
        numeric_value: row.numeric_value,
        is_enabled: row.is_enabled,
        sort_order: row.sort_order,
        tenant_id: row.tenant_id,
        tenant_name: row.tenant_name.clone(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        i18n: HashMap::new(),
    }
}

fn to_i18n_dto(row: &crate::repos::dict_entry::DictEntryI18nRow) -> DictEntryI18nDto {
    DictEntryI18nDto {
        entry_label: row.entry_label.clone().unwrap_or_default(),
        description: row.description.clone(),
        language_code: row.language_code.clone(),
        language_name: None,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictEntryData {
    #[serde(default)]
    pub type_id: Option<i64>,
    #[serde(default)]
    pub entry_value: Option<String>,
    #[serde(default)]
    pub numeric_value: Option<i64>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
    #[serde(default)]
    pub i18n: HashMap<String, DictEntryI18nData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictEntryI18nData {
    #[serde(default)]
    pub entry_label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDictEntryBody {
    #[serde(default)]
    pub data: Option<DictEntryData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldMask {
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDictEntryBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<DictEntryData>,
    #[serde(default)]
    pub update_mask: Option<FieldMask>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 删除入参：前端 DELETE /dict/entries?ids=1&ids=2（repeated query param）
#[derive(Debug, Deserialize)]
pub struct DeleteIdsQuery {
    #[serde(default)]
    pub ids: Vec<i64>,
}

/// Go 端校验：entry_value 必填（ent NotEmpty）。
fn validate_data(data: &DictEntryData) -> Result<String, AppError> {
    let value = data.entry_value.as_deref().map(str::trim).unwrap_or("");
    if value.is_empty() {
        return Err(AppError::Validation("dict entry value is required".into()));
    }
    Ok(value.to_string())
}

fn to_i18n_inputs(data: &DictEntryData) -> HashMap<String, I18nInput> {
    data.i18n
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                I18nInput {
                    entry_label: v.entry_label.clone(),
                    description: v.description.clone(),
                },
            )
        })
        .collect()
}

pub async fn dict_entry_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, DICT_ENTRY_COLUMNS)?;
    let repo = DictEntryRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        // 编译时把列名加 e. 前缀，避免与 join 表的同名列歧义
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("e.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, DICT_ENTRY_COLUMNS)?;
        order_parts.push(format!("\"e\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        let mut dto = to_dto(row);
        dto.i18n = load_i18n_map(&repo, row.id).await?;
        items.push(dto);
    }
    Ok(json_ok(ListResponse::new(items, total)))
}

pub async fn dict_entry_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateDictEntryBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let entry_value = validate_data(&data)?;

    let repo = DictEntryRepo::new(crate::handlers::script::db_of(&state)?);
    let tenant_id = data.tenant_id.or_else(|| {
        if operator.tenant_id > 0 {
            Some(operator.tenant_id)
        } else {
            None
        }
    });
    let entry_id = repo
        .create(
            &entry_value,
            data.numeric_value,
            data.type_id,
            data.is_enabled,
            data.sort_order,
            tenant_id,
            operator.user_id,
        )
        .await?;
    if !data.i18n.is_empty() {
        repo.i18n_replace_by_entry_id(entry_id, tenant_id, operator.user_id, &to_i18n_inputs(&data))
            .await?;
    }
    Ok(json_empty())
}

pub async fn dict_entry_delete(
    State(state): State<AppState>,
    Query(params): Query<DeleteIdsQuery>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    if params.ids.is_empty() {
        return Err(AppError::Validation("invalid parameter".into()));
    }
    let repo = DictEntryRepo::new(crate::handlers::script::db_of(&state)?);
    let _ = repo.batch_delete(&params.ids).await?;
    Ok(json_empty())
}

pub async fn dict_entry_list_by_type_code(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let type_code = params
        .get("typeCode")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let local = params.get("local").map(|s| s.to_string()).filter(|s| !s.is_empty());

    let repo = DictEntryRepo::new(crate::handlers::script::db_of(&state)?);
    let rows = repo.list_by_type_code(&type_code).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        let mut dto = to_dto(row);
        dto.i18n = match &local {
            Some(loc) => {
                // 对齐 Go：仅填充 local 单条目 map；local 无翻译时留空（Go 端 .Only() 会 500，这里宽松处理）
                let mut m = HashMap::new();
                if let Some(i18n_row) = repo.i18n_get_by_entry_id_and_lang(row.id, loc).await? {
                    m.insert(loc.clone(), to_i18n_dto(&i18n_row));
                }
                m
            }
            None => load_i18n_map(&repo, row.id).await?,
        };
        items.push(dto);
    }
    Ok(json_ok(ByTypeCodeResponse { items }))
}

pub async fn dict_entry_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateDictEntryBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = DictEntryRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        let entry_value = validate_data(&data)?;
        let tenant_id = data.tenant_id.or_else(|| {
            if operator.tenant_id > 0 {
                Some(operator.tenant_id)
            } else {
                None
            }
        });
        let entry_id = repo
            .create(
                &entry_value,
                data.numeric_value,
                data.type_id,
                data.is_enabled,
                data.sort_order,
                tenant_id,
                operator.user_id,
            )
            .await?;
        if !data.i18n.is_empty() {
            repo.i18n_replace_by_entry_id(entry_id, tenant_id, operator.user_id, &to_i18n_inputs(&data))
                .await?;
        }
        return Ok(json_empty());
    }

    let entry_value = data.entry_value.as_deref().map(str::trim);
    if let Some(v) = entry_value {
        if v.is_empty() {
            return Err(AppError::Validation("dict entry value is required".into()));
        }
    }

    let updated = repo
        .update(
            id,
            entry_value,
            data.numeric_value,
            data.is_enabled,
            data.sort_order,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("dict entry not found".into()));
    }

    // i18n 仅当 updateMask 显式包含 i18n 且非空时替换（对齐 Go：hasI18n && len>0）
    let has_i18n = body
        .update_mask
        .as_ref()
        .map(|m| m.paths.iter().any(|p| p.eq_ignore_ascii_case("i18n")))
        .unwrap_or(false);
    if has_i18n && !data.i18n.is_empty() {
        let tenant_id = data.tenant_id.or_else(|| {
            if operator.tenant_id > 0 {
                Some(operator.tenant_id)
            } else {
                None
            }
        });
        repo.i18n_replace_by_entry_id(id, tenant_id, operator.user_id, &to_i18n_inputs(&data))
            .await?;
    }
    Ok(json_empty())
}

/// 按条目 ID 加载完整 i18n map（key 为语言代码）
async fn load_i18n_map(
    repo: &DictEntryRepo,
    entry_id: i64,
) -> Result<HashMap<String, DictEntryI18nDto>, AppError> {
    let rows = repo.i18n_list_by_entry_id(entry_id).await?;
    let mut m = HashMap::with_capacity(rows.len());
    for r in &rows {
        if let Some(code) = &r.language_code {
            m.insert(code.clone(), to_i18n_dto(r));
        }
    }
    Ok(m)
}

/// by-type-code 响应：{items:[...]}（无 total，对齐 Go ListDictEntryByTypeCodeResponse）
#[derive(Debug, Serialize)]
pub struct ByTypeCodeResponse {
    pub items: Vec<DictEntryDto>,
}
