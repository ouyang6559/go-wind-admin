// plan repo：sys_plans CRUD SQL（对齐 Go plan_repo.go）。
// 枚举按 proto json 名存储（version: FREE/STANDARD/ENTERPRISE；
// expiry_policy: READONLY/BLOCK_LOGIN/FREEZE），与 Go 端 EnumTypeConverter 写库值一致；
// list/get 过滤 deleted_at is null，Delete 为硬删除（TimeAt mixin 无软删拦截器）。

use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct PlanRow {
    pub id: i64,
    pub name: Option<String>,
    /// proto 枚举名（FREE/STANDARD/ENTERPRISE）
    pub version: Option<String>,
    /// proto 枚举名（READONLY/BLOCK_LOGIN/FREEZE）
    pub expiry_policy: Option<String>,
    pub data_retention_days: Option<i64>,
    pub description: Option<String>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

const SELECT_COLS: &str = "id, name, version, expiry_policy, data_retention_days, description, \
                           remark, created_by, updated_by, deleted_by, \
                           to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

type PlanTuple = (
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn map_row(r: PlanTuple) -> PlanRow {
    PlanRow {
        id: r.0,
        name: r.1,
        version: r.2,
        expiry_policy: r.3,
        data_retention_days: r.4,
        description: r.5,
        remark: r.6,
        created_by: r.7,
        updated_by: r.8,
        deleted_by: r.9,
        created_at: r.10,
        updated_at: r.11,
        deleted_at: r.12,
    }
}

pub struct PlanRepo {
    pub db: AnyPool,
}

impl PlanRepo {
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
    ) -> Result<(Vec<PlanRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_plans where deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count plans failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} from sys_plans where deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query_as::<sqlx::Any, PlanTuple>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list plans failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok((rows.into_iter().map(map_row).collect(), total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<PlanRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} from sys_plans where id = $1 and deleted_at is null limit 1"
        );
        let row = sqlx::query_as::<sqlx::Any, PlanTuple>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get plan failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    /// 名称唯一性检查（exclude_id 排除自身）
    pub async fn name_exists(&self, name: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_plans where name = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check plan name failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建套餐。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        version: Option<&str>,
        expiry_policy: Option<&str>,
        data_retention_days: Option<i64>,
        description: Option<&str>,
        remark: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_plans \
                   (name, version, expiry_policy, data_retention_days, description, remark, \
                    created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $8::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(version)
            .bind(expiry_policy)
            .bind(data_retention_days)
            .bind(description)
            .bind(remark)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert plan failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新套餐：仅非 None 字段。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        version: Option<&str>,
        expiry_policy: Option<&str>,
        data_retention_days: Option<i64>,
        description: Option<&str>,
        remark: Option<&str>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / ""
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = name {
            push!("name", v.to_string(), "");
        }
        if let Some(v) = version {
            push!("version", v.to_string(), "");
        }
        if let Some(v) = expiry_policy {
            push!("expiry_policy", v.to_string(), "");
        }
        if let Some(v) = data_retention_days {
            push!("data_retention_days", v.to_string(), "::int8");
        }
        if let Some(v) = description {
            push!("description", v.to_string(), "");
        }
        if let Some(v) = remark {
            push!("remark", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_plans set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update plan failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除（ids 批量，对齐 DELETE /plans 集合路径）。
    pub async fn delete_ids(&self, ids: &[i64]) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let placeholders = (1..=ids.len())
            .map(|i| format!("${i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("delete from sys_plans where id in ({placeholders})");
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "delete plans failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }
}
