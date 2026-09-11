// position repo：sys_positions CRUD SQL（对齐 Go position_repo.go）。
// 枚举按 proto json 名存储（ON/OFF、REGULAR/LEADER/MANAGER/INTERN/CONTRACT/OTHER），
// 与 Go 端 EnumTypeConverter 写库值一致；列表/查询过滤 deleted_at is null，
// Delete 为硬删除（Go 端 TimeAt mixin 无软删拦截器）。

use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct PositionRow {
    pub id: i64,
    pub name: Option<String>,
    pub code: Option<String>,
    pub headcount: Option<i64>,
    pub sort_order: Option<i64>,
    /// proto 枚举名（ON/OFF）
    pub status: Option<String>,
    /// proto 枚举名（REGULAR/LEADER/MANAGER/INTERN/CONTRACT/OTHER）
    pub r#type: Option<String>,
    pub remark: Option<String>,
    pub description: Option<String>,
    pub job_family: Option<String>,
    pub job_grade: Option<String>,
    pub level: Option<i64>,
    pub is_key_position: Option<bool>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub org_unit_id: Option<i64>,
    pub org_unit_name: Option<String>,
    pub reports_to_position_id: Option<i64>,
    pub reports_to_position_name: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（p.* + 关联显示名）
const SELECT_COLS: &str = "p.id, p.name, p.code, p.headcount, p.sort_order, p.status, p.type, \
                           p.remark, p.description, p.job_family, p.job_grade, p.level, \
                           p.is_key_position, p.tenant_id, t.name as tenant_name, \
                           p.org_unit_id, ou.name as org_unit_name, \
                           p.reports_to_position_id, rp.name as reports_to_position_name, \
                           to_char(p.start_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as start_at, \
                           to_char(p.end_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as end_at, \
                           p.created_by, p.updated_by, p.deleted_by, \
                           to_char(p.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(p.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(p.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_positions p \
                    left join sys_org_units ou on ou.id = p.org_unit_id and ou.deleted_at is null \
                    left join sys_tenants t on t.id = p.tenant_id and t.deleted_at is null \
                    left join sys_positions rp on rp.id = p.reports_to_position_id and rp.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<PositionRow, AppError> {
    Ok(PositionRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "position row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        code: row.try_get::<Option<String>, _>("code").ok().flatten(),
        headcount: row.try_get::<Option<i64>, _>("headcount").ok().flatten(),
        sort_order: row.try_get::<Option<i64>, _>("sort_order").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        r#type: row.try_get::<Option<String>, _>("type").ok().flatten(),
        remark: row.try_get::<Option<String>, _>("remark").ok().flatten(),
        description: row.try_get::<Option<String>, _>("description").ok().flatten(),
        job_family: row.try_get::<Option<String>, _>("job_family").ok().flatten(),
        job_grade: row.try_get::<Option<String>, _>("job_grade").ok().flatten(),
        level: row.try_get::<Option<i64>, _>("level").ok().flatten(),
        is_key_position: row
            .try_get::<Option<bool>, _>("is_key_position")
            .ok()
            .flatten(),
        tenant_id: row.try_get::<Option<i64>, _>("tenant_id").ok().flatten(),
        tenant_name: row.try_get::<Option<String>, _>("tenant_name").ok().flatten(),
        org_unit_id: row.try_get::<Option<i64>, _>("org_unit_id").ok().flatten(),
        org_unit_name: row.try_get::<Option<String>, _>("org_unit_name").ok().flatten(),
        reports_to_position_id: row
            .try_get::<Option<i64>, _>("reports_to_position_id")
            .ok()
            .flatten(),
        reports_to_position_name: row
            .try_get::<Option<String>, _>("reports_to_position_name")
            .ok()
            .flatten(),
        start_at: row.try_get::<Option<String>, _>("start_at").ok().flatten(),
        end_at: row.try_get::<Option<String>, _>("end_at").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    })
}

pub struct PositionRepo {
    pub db: AnyPool,
}

impl PositionRepo {
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
    ) -> Result<(Vec<PositionRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_positions p where p.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count positions failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "p.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where p.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list positions failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<PositionRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where p.id = $1 and p.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get position failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 名称唯一性检查（exclude_id 排除自身）
    pub async fn name_exists(&self, name: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_positions where name = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check position name failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 编码唯一性检查（租户内 (tenant_id, code) 唯一，Rust 端以 code 全局近似）
    pub async fn code_exists(&self, code: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_positions where code = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(code)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check position code failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建职位。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        code: &str,
        tenant_id: Option<i64>,
        org_unit_id: Option<i64>,
        reports_to_position_id: Option<i64>,
        sort_order: Option<i64>,
        status: Option<&str>,
        r#type: Option<&str>,
        job_family: Option<&str>,
        job_grade: Option<&str>,
        level: Option<i64>,
        is_key_position: Option<bool>,
        headcount: Option<i64>,
        description: Option<&str>,
        remark: Option<&str>,
        start_at: Option<&str>,
        end_at: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_positions \
                   (name, code, tenant_id, org_unit_id, reports_to_position_id, sort_order, status, type, \
                    job_family, job_grade, level, is_key_position, headcount, description, remark, \
                    start_at, end_at, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, \
                           $16::timestamptz, $17::timestamptz, $18, $19::timestamptz, $19::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(code)
            .bind(tenant_id)
            .bind(org_unit_id)
            .bind(reports_to_position_id)
            .bind(sort_order)
            .bind(status)
            .bind(r#type)
            .bind(job_family)
            .bind(job_grade)
            .bind(level)
            .bind(is_key_position)
            .bind(headcount)
            .bind(description)
            .bind(remark)
            .bind(start_at)
            .bind(end_at)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert position failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新职位：仅非 None 字段。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        code: Option<&str>,
        org_unit_id: Option<i64>,
        reports_to_position_id: Option<i64>,
        sort_order: Option<i64>,
        status: Option<&str>,
        r#type: Option<&str>,
        job_family: Option<&str>,
        job_grade: Option<&str>,
        level: Option<i64>,
        is_key_position: Option<bool>,
        headcount: Option<i64>,
        description: Option<&str>,
        remark: Option<&str>,
        start_at: Option<&str>,
        end_at: Option<&str>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::bool" / "::timestamptz" / ""
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
        if let Some(v) = org_unit_id {
            push!("org_unit_id", v.to_string(), "::int8");
        }
        if let Some(v) = reports_to_position_id {
            push!("reports_to_position_id", v.to_string(), "::int8");
        }
        if let Some(v) = sort_order {
            push!("sort_order", v.to_string(), "::int8");
        }
        if let Some(v) = status {
            push!("status", v.to_string(), "");
        }
        if let Some(v) = r#type {
            push!("type", v.to_string(), "");
        }
        if let Some(v) = job_family {
            push!("job_family", v.to_string(), "");
        }
        if let Some(v) = job_grade {
            push!("job_grade", v.to_string(), "");
        }
        if let Some(v) = level {
            push!("level", v.to_string(), "::int8");
        }
        if let Some(v) = is_key_position {
            push!("is_key_position", v.to_string(), "::bool");
        }
        if let Some(v) = headcount {
            push!("headcount", v.to_string(), "::int8");
        }
        if let Some(v) = description {
            push!("description", v.to_string(), "");
        }
        if let Some(v) = remark {
            push!("remark", v.to_string(), "");
        }
        if let Some(v) = start_at {
            push!("start_at", v.to_string(), "::timestamptz");
        }
        if let Some(v) = end_at {
            push!("end_at", v.to_string(), "::timestamptz");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_positions set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update position failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let sql = "delete from sys_positions where id = $1 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete position failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}
