// plan_module repo：sys_plan_modules CRUD SQL（对齐 Go plan_module_repo.go）。
// module 为 proto 枚举名字符串（DASHBOARD/OPM/SYSTEM/DICT/TENANT/PERMISSION/LOG/
// INTERNAL_MESSAGE/FILE/TASK）；plan_id 为 edge 生成的外键列；
// list/get 过滤 deleted_at is null，Delete 为硬删除。

use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct PlanModuleRow {
    pub id: i64,
    pub plan_id: Option<i64>,
    /// proto 枚举名（DASHBOARD/OPM/SYSTEM/DICT/TENANT/PERMISSION/LOG/INTERNAL_MESSAGE/FILE/TASK）
    pub module: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

const SELECT_COLS: &str = "id, plan_id, module, created_by, updated_by, deleted_by, \
                           to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

type PlanModuleTuple = (
    i64,
    Option<i64>,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn map_row(r: PlanModuleTuple) -> PlanModuleRow {
    PlanModuleRow {
        id: r.0,
        plan_id: r.1,
        module: r.2,
        created_by: r.3,
        updated_by: r.4,
        deleted_by: r.5,
        created_at: r.6,
        updated_at: r.7,
        deleted_at: r.8,
    }
}

pub struct PlanModuleRepo {
    pub db: AnyPool,
}

impl PlanModuleRepo {
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
    ) -> Result<(Vec<PlanModuleRow>, u64), AppError> {
        let total_sql = format!(
            "select count(*) from sys_plan_modules where deleted_at is null{where_clause}"
        );
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count plan modules failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} from sys_plan_modules where deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query_as::<sqlx::Any, PlanModuleTuple>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list plan modules failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok((rows.into_iter().map(map_row).collect(), total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<PlanModuleRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} from sys_plan_modules where id = $1 and deleted_at is null limit 1"
        );
        let row = sqlx::query_as::<sqlx::Any, PlanModuleTuple>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get plan module failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    /// 创建白名单项。
    pub async fn create(
        &self,
        plan_id: Option<i64>,
        module: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_plan_modules \
                   (plan_id, module, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4::timestamptz, $4::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(plan_id)
            .bind(module)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert plan module failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新白名单项：仅非 None 字段。
    pub async fn update(
        &self,
        id: i64,
        plan_id: Option<i64>,
        module: Option<&str>,
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

        if let Some(v) = plan_id {
            push!("plan_id", v.to_string(), "::int8");
        }
        if let Some(v) = module {
            push!("module", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_plan_modules set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update plan module failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除（ids 批量，对齐 DELETE /plan-modules 集合路径）。
    pub async fn delete_ids(&self, ids: &[i64]) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let placeholders = (1..=ids.len())
            .map(|i| format!("${i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("delete from sys_plan_modules where id in ({placeholders})");
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "delete plan modules failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }
}
