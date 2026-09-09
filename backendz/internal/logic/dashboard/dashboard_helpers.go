package dashboard

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/loginauditlog"
	"go-wind-admin/backendz/internal/ent/gen/operationauditlog"
	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/ent/gen/user"
)

// distributionRow 是按字段分组聚合的扫描结构，与滑过群组列 Action/Status 共用。
type distributionRow struct {
	Action string `sql:"action"`
	Status string `sql:"status"`
	Count  int    `sql:"count"`
}

// trendRow 登录趋势按日分桶结果项。
type trendRow struct {
	Date  string
	Count int
}

// startOfToday 返回今日零点（本地时区）。
func startOfToday() time.Time {
	now := time.Now()
	return time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, now.Location())
}

// countActiveUsers 统计用户总数。
func countActiveUsers(ctx context.Context, client *gen.Client) (int, error) {
	return client.User.Query().Where(user.DeletedAtIsNil()).Count(ctx)
}

// countRoles 统计角色总数。
func countRoles(ctx context.Context, client *gen.Client) (int, error) {
	return client.Role.Query().Where(role.DeletedAtIsNil()).Count(ctx)
}

// countTodayLogins 统计今日登录次数（action_type=LOGIN 且 created_at>=今日零点）。
func countTodayLogins(ctx context.Context, client *gen.Client) (int, error) {
	today := startOfToday()
	return client.LoginAuditLog.Query().
		Where(
			loginauditlog.ActionTypeEQ(loginauditlog.ActionTypeLogin),
			loginauditlog.CreatedAtGTE(today),
		).Count(ctx)
}

// countTodayOperations 统计今日操作审计条数。
func countTodayOperations(ctx context.Context, client *gen.Client) (int, error) {
	return client.OperationAuditLog.Query().
		Where(operationauditlog.CreatedAtGTE(startOfToday())).
		Count(ctx)
}

// loginTrend 统计近 days 天每日登录次数，按日期升序、缺日补零。
func loginTrend(ctx context.Context, client *gen.Client, days int) ([]trendRow, error) {
	if days <= 0 {
		days = 7
	}
	today := startOfToday()
	start := today.AddDate(0, 0, -(days - 1))

	type onlyCreated struct {
		CreatedAt time.Time `sql:"created_at"`
	}
	var logs []onlyCreated
	if err := client.LoginAuditLog.Query().
		Where(
			loginauditlog.ActionTypeEQ(loginauditlog.ActionTypeLogin),
			loginauditlog.CreatedAtGTE(start),
		).Select(loginauditlog.FieldCreatedAt).Scan(ctx, &logs); err != nil {
		return nil, err
	}

	loc := today.Location()
	buckets := make([]trendRow, 0, days)
	idx := make(map[string]int, days)
	for i := 0; i < days; i++ {
		d := start.AddDate(0, 0, i).Format("2006-01-02")
		idx[d] = i
		buckets = append(buckets, trendRow{Date: d, Count: 0})
	}
	for _, lg := range logs {
		d := lg.CreatedAt.In(loc).Format("2006-01-02")
		if i, ok := idx[d]; ok {
			buckets[i].Count++
		}
	}
	return buckets, nil
}

// operationActionDistribution 按操作 action 分组统计。
func operationActionDistribution(ctx context.Context, client *gen.Client) ([]distributionRow, error) {
	var rows []distributionRow
	err := client.OperationAuditLog.Query().
		GroupBy(operationauditlog.FieldAction).
		Aggregate(gen.As(gen.Count(), "count")).
		Scan(ctx, &rows)
	return rows, err
}

// loginStatusDistribution 按登录 status 分组统计。
func loginStatusDistribution(ctx context.Context, client *gen.Client) ([]distributionRow, error) {
	var rows []distributionRow
	err := client.LoginAuditLog.Query().
		GroupBy(loginauditlog.FieldStatus).
		Aggregate(gen.As(gen.Count(), "count")).
		Scan(ctx, &rows)
	return rows, err
}