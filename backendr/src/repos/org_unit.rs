// org_unit repo：sys_org_units CRUD SQL（对齐 Go org_unit_repo.go）。
// 树形表：parent_id + path（Tree/TreePath mixin）；Delete 递归删除全部后代
// （对齐 Go QueryAllChildrenIds）；business_scopes / permission_tags 为 jsonb 数组列，
// SELECT 用 ::text 转出后解析，写入用 $N::jsonb。
// 枚举按 proto json 名存储（status: ON/OFF；type: COMPANY/DIVISION/...），
// 与 Go 端 EnumTypeConverter 写库值一致；list/get 过滤 deleted_at is null。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct OrgUnitRow {
    pub id: i64,
    pub name: Option<String>,
    pub code: Option<String>,
    pub leader_id: Option<i64>,
    pub leader_name: Option<String>,
    /// proto 枚举名（COMPANY/DIVISION/DEPARTMENT/TEAM/PROJECT/COMMITTEE/REGION/SUBSIDIARY/BRANCH/OTHER）
    pub r#type: Option<String>,
    pub business_scopes: Option<Vec<String>>,
    pub external_id: Option<String>,
    pub is_legal_entity: Option<bool>,
    pub registration_number: Option<String>,
    pub tax_id: Option<String>,
    pub legal_entity_org_id: Option<i64>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub timezone: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub contact_user_id: Option<i64>,
    pub contact_user_name: Option<String>,
    pub permission_tags: Option<Vec<String>>,
    /// proto 枚举名（ON/OFF）
    pub status: Option<String>,
    pub sort_order: Option<i64>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub remark: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
    pub path: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（ou.* + 关联显示名）
const SELECT_COLS: &str = "ou.id, ou.name, ou.code, ou.leader_id, lu.username as leader_name, \
                           ou.type, ou.business_scopes::text as business_scopes, ou.external_id, \
                           ou.is_legal_entity, ou.registration_number, ou.tax_id, \
                           ou.legal_entity_org_id, ou.address, ou.phone, ou.email, ou.timezone, \
                           ou.country, ou.latitude, ou.longitude, \
                           to_char(ou.start_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as start_at, \
                           to_char(ou.end_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as end_at, \
                           ou.contact_user_id, cu.username as contact_user_name, \
                           ou.permission_tags::text as permission_tags, ou.status, ou.sort_order, \
                           ou.tenant_id, t.name as tenant_name, ou.remark, ou.description, \
                           ou.parent_id, ou.path, \
                           ou.created_by, ou.updated_by, ou.deleted_by, \
                           to_char(ou.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(ou.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(ou.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_org_units ou \
                    left join sys_tenants t on t.id = ou.tenant_id and t.deleted_at is null \
                    left join sys_users lu on lu.id = ou.leader_id and lu.deleted_at is null \
                    left join sys_users cu on cu.id = ou.contact_user_id and cu.deleted_at is null";

/// jsonb 数组文本（如 `["a","b"]`）解析为 Vec<String>；非法/空按 None 处理
fn parse_str_array(raw: Option<String>) -> Option<Vec<String>> {
    raw.and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
}

fn map_row(row: &sqlx::any::AnyRow) -> Result<OrgUnitRow, AppError> {
    let mut r = OrgUnitRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "org unit row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        code: row.try_get::<Option<String>, _>("code").ok().flatten(),
        leader_id: row.try_get::<Option<i64>, _>("leader_id").ok().flatten(),
        leader_name: row.try_get::<Option<String>, _>("leader_name").ok().flatten(),
        r#type: row.try_get::<Option<String>, _>("type").ok().flatten(),
        business_scopes: parse_str_array(
            row.try_get::<Option<String>, _>("business_scopes").ok().flatten(),
        ),
        external_id: row.try_get::<Option<String>, _>("external_id").ok().flatten(),
        is_legal_entity: row
            .try_get::<Option<bool>, _>("is_legal_entity")
            .ok()
            .flatten(),
        registration_number: row
            .try_get::<Option<String>, _>("registration_number")
            .ok()
            .flatten(),
        tax_id: row.try_get::<Option<String>, _>("tax_id").ok().flatten(),
        legal_entity_org_id: row
            .try_get::<Option<i64>, _>("legal_entity_org_id")
            .ok()
            .flatten(),
        address: row.try_get::<Option<String>, _>("address").ok().flatten(),
        phone: row.try_get::<Option<String>, _>("phone").ok().flatten(),
        email: row.try_get::<Option<String>, _>("email").ok().flatten(),
        timezone: row.try_get::<Option<String>, _>("timezone").ok().flatten(),
        country: row.try_get::<Option<String>, _>("country").ok().flatten(),
        latitude: row.try_get::<Option<f64>, _>("latitude").ok().flatten(),
        longitude: row.try_get::<Option<f64>, _>("longitude").ok().flatten(),
        start_at: row.try_get::<Option<String>, _>("start_at").ok().flatten(),
        end_at: row.try_get::<Option<String>, _>("end_at").ok().flatten(),
        contact_user_id: row
            .try_get::<Option<i64>, _>("contact_user_id")
            .ok()
            .flatten(),
        contact_user_name: row
            .try_get::<Option<String>, _>("contact_user_name")
            .ok()
            .flatten(),
        permission_tags: parse_str_array(
            row.try_get::<Option<String>, _>("permission_tags").ok().flatten(),
        ),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        sort_order: row.try_get::<Option<i64>, _>("sort_order").ok().flatten(),
        tenant_id: row.try_get::<Option<i64>, _>("tenant_id").ok().flatten(),
        tenant_name: row.try_get::<Option<String>, _>("tenant_name").ok().flatten(),
        remark: row.try_get::<Option<String>, _>("remark").ok().flatten(),
        description: row.try_get::<Option<String>, _>("description").ok().flatten(),
        parent_id: row.try_get::<Option<i64>, _>("parent_id").ok().flatten(),
        path: row.try_get::<Option<String>, _>("path").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    };
    // business_scopes/permission_tags 为 NULL 或解析失败时保持 None（省略输出）
    if r.business_scopes.as_deref() == Some(&[]) {
        r.business_scopes = None;
    }
    if r.permission_tags.as_deref() == Some(&[]) {
        r.permission_tags = None;
    }
    Ok(r)
}

/// 计算树路径（对齐 Go entCrud.ComputeTreePath）
pub fn compute_tree_path(parent_path: &str, node_id: i64) -> String {
    if parent_path.is_empty() {
        return "/".to_string();
    }
    let mut pp = parent_path.to_string();
    if !pp.ends_with('/') {
        pp.push('/');
    }
    format!("{pp}{node_id}/")
}

/// Vec<String> → jsonb 文本（`["a","b"]`）
fn to_json_array(v: Option<&Vec<String>>) -> Option<String> {
    v.map(|items| serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string()))
}

pub struct OrgUnitRepo {
    pub db: AnyPool,
}

impl OrgUnitRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    pub async fn list(
        &self,
        offset: u64,
        limit: u64,
        where_clause: &str,
        params: &[String],
        order_by: &str,
    ) -> Result<(Vec<OrgUnitRow>, u64), AppError> {
        let total_sql = format!(
            "select count(*) from sys_org_units ou where ou.deleted_at is null{where_clause}"
        );
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count org units failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "ou.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where ou.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list org units failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<OrgUnitRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where ou.id = $1 and ou.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get org unit failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 名称唯一性检查（同级 tenant+parent 内唯一；Rust 端以全局近似）
    pub async fn name_exists(&self, name: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql =
            "select 1 from sys_org_units where name = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check org unit name failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 编码唯一性检查（租户内 code 唯一，Rust 端以全局近似）
    pub async fn code_exists(&self, code: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql =
            "select 1 from sys_org_units where code = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(code)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check org unit code failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建组织单元；返回 (id, path)。
    /// 先插入（不含 path），再按 parent.path 计算树路径回填（对齐 Go setTreePath）。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        code: Option<&str>,
        leader_id: Option<i64>,
        r#type: Option<&str>,
        business_scopes: Option<&Vec<String>>,
        external_id: Option<&str>,
        is_legal_entity: Option<bool>,
        registration_number: Option<&str>,
        tax_id: Option<&str>,
        legal_entity_org_id: Option<i64>,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
        timezone: Option<&str>,
        country: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        start_at: Option<&str>,
        end_at: Option<&str>,
        contact_user_id: Option<i64>,
        permission_tags: Option<&Vec<String>>,
        status: Option<&str>,
        sort_order: Option<i64>,
        tenant_id: Option<i64>,
        remark: Option<&str>,
        description: Option<&str>,
        parent_id: Option<i64>,
        created_by: i64,
    ) -> Result<(i64, String), AppError> {
        let sql = "insert into sys_org_units \
                   (name, code, leader_id, type, business_scopes, external_id, is_legal_entity, \
                    registration_number, tax_id, legal_entity_org_id, address, phone, email, \
                    timezone, country, latitude, longitude, start_at, end_at, contact_user_id, \
                    permission_tags, status, sort_order, tenant_id, remark, description, parent_id, \
                    created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5::jsonb, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, \
                           $16, $17, $18::timestamptz, $19::timestamptz, $20, $21::jsonb, $22, $23, \
                           $24, $25, $26, $27, $28, $29::timestamptz, $29::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(code)
            .bind(leader_id)
            .bind(r#type)
            .bind(to_json_array(business_scopes))
            .bind(external_id)
            .bind(is_legal_entity)
            .bind(registration_number)
            .bind(tax_id)
            .bind(legal_entity_org_id)
            .bind(address)
            .bind(phone)
            .bind(email)
            .bind(timezone)
            .bind(country)
            .bind(latitude)
            .bind(longitude)
            .bind(start_at)
            .bind(end_at)
            .bind(contact_user_id)
            .bind(to_json_array(permission_tags))
            .bind(status)
            .bind(sort_order)
            .bind(tenant_id)
            .bind(remark)
            .bind(description)
            .bind(parent_id)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert org unit failed".into(),
                source: Some(Box::new(e)),
            })?;

        // 计算并回填树路径（对齐 Go setTreePath：parentPath + "/{id}/"）
        let parent_path: String = if let Some(pid) = parent_id {
            let sql = "select path from sys_org_units where id = $1 and deleted_at is null limit 1";
            let q: Option<(Option<String>,)> = sqlx::query_as::<sqlx::Any, (Option<String>,)>(sql)
                .bind(pid)
                .fetch_optional(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "query org unit parent path failed".into(),
                    source: Some(Box::new(e)),
                })?;
            q.and_then(|r| r.0).unwrap_or_default()
        } else {
            String::new()
        };
        let path = compute_tree_path(&parent_path, row.0);
        let _ = sqlx::query::<sqlx::Any>("update sys_org_units set path = $1 where id = $2")
            .bind(&path)
            .bind(row.0)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "update org unit path failed".into(),
                source: Some(Box::new(e)),
            })?;

        Ok((row.0, path))
    }

    /// 更新组织单元：仅非 None 字段。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        code: Option<&str>,
        leader_id: Option<i64>,
        r#type: Option<&str>,
        business_scopes: Option<&Vec<String>>,
        external_id: Option<&str>,
        is_legal_entity: Option<bool>,
        registration_number: Option<&str>,
        tax_id: Option<&str>,
        legal_entity_org_id: Option<i64>,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
        timezone: Option<&str>,
        country: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        start_at: Option<&str>,
        end_at: Option<&str>,
        contact_user_id: Option<i64>,
        permission_tags: Option<&Vec<String>>,
        status: Option<&str>,
        sort_order: Option<i64>,
        remark: Option<&str>,
        description: Option<&str>,
        parent_id: Option<i64>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::bool" / "::timestamptz" / "::jsonb" / ""
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = name {
            push!("name", v.to_string(), "");
        }
        if let Some(v) = code {
            push!("code", v.to_string(), "");
        }
        if let Some(v) = leader_id {
            push!("leader_id", v.to_string(), "::int8");
        }
        if let Some(v) = r#type {
            push!("type", v.to_string(), "");
        }
        if let Some(v) = business_scopes {
            if let Some(json) = to_json_array(Some(v)) {
                push!("business_scopes", json, "::jsonb");
            }
        }
        if let Some(v) = external_id {
            push!("external_id", v.to_string(), "");
        }
        if let Some(v) = is_legal_entity {
            push!("is_legal_entity", v.to_string(), "::bool");
        }
        if let Some(v) = registration_number {
            push!("registration_number", v.to_string(), "");
        }
        if let Some(v) = tax_id {
            push!("tax_id", v.to_string(), "");
        }
        if let Some(v) = legal_entity_org_id {
            push!("legal_entity_org_id", v.to_string(), "::int8");
        }
        if let Some(v) = address {
            push!("address", v.to_string(), "");
        }
        if let Some(v) = phone {
            push!("phone", v.to_string(), "");
        }
        if let Some(v) = email {
            push!("email", v.to_string(), "");
        }
        if let Some(v) = timezone {
            push!("timezone", v.to_string(), "");
        }
        if let Some(v) = country {
            push!("country", v.to_string(), "");
        }
        if let Some(v) = latitude {
            push!("latitude", v.to_string(), "");
        }
        if let Some(v) = longitude {
            push!("longitude", v.to_string(), "");
        }
        if let Some(v) = start_at {
            push!("start_at", v.to_string(), "::timestamptz");
        }
        if let Some(v) = end_at {
            push!("end_at", v.to_string(), "::timestamptz");
        }
        if let Some(v) = contact_user_id {
            push!("contact_user_id", v.to_string(), "::int8");
        }
        if let Some(v) = permission_tags {
            if let Some(json) = to_json_array(Some(v)) {
                push!("permission_tags", json, "::jsonb");
            }
        }
        if let Some(v) = status {
            push!("status", v.to_string(), "");
        }
        if let Some(v) = sort_order {
            push!("sort_order", v.to_string(), "::int8");
        }
        if let Some(v) = remark {
            push!("remark", v.to_string(), "");
        }
        if let Some(v) = description {
            push!("description", v.to_string(), "");
        }
        if let Some(v) = parent_id {
            push!("parent_id", v.to_string(), "::int8");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_org_units set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update org unit failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除：递归收集该节点及全部后代后批量删除（对齐 Go QueryAllChildrenIds）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let mut ids: Vec<i64> = vec![id];
        let mut stack: Vec<i64> = vec![id];
        while let Some(pid) = stack.pop() {
            let sql =
                "select id from sys_org_units where parent_id = $1 and deleted_at is null";
            let rows: Vec<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
                .bind(pid)
                .fetch_all(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "query org unit children failed".into(),
                    source: Some(Box::new(e)),
                })?;
            for (cid,) in rows {
                ids.push(cid);
                stack.push(cid);
            }
        }
        let placeholders = (1..=ids.len())
            .map(|i| format!("${i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("delete from sys_org_units where id in ({placeholders})");
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for i in &ids {
            q = q.bind(i);
        }
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "delete org units failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }
}
