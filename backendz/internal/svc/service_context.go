// Code scaffolded by goctl. Safe to edit.

package svc

import (
	"context"
	"database/sql"
	"fmt"
	"strings"
	"time"

	"entgo.io/ent/dialect"
	sqlDriver "entgo.io/ent/dialect/sql"
	sqlpgx "github.com/jackc/pgx/v5/stdlib"
	"github.com/zeromicro/go-zero/core/stores/redis"

	"go-wind-admin/backendz/internal/config"
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/migrate"
	"go-wind-admin/backendz/internal/pkg/session"
	"go-wind-admin/backendz/internal/pkg/ssehub"
	"go-wind-admin/backendz/internal/pkg/token"
)

// init 将 pgx 驱动同时注册到 "postgres" 名下，使 ent sql 能按 dialect 名打开。
// pgx/v5/stdlib 默认只注册 "pgx"；ent 的 sql.Open 按 dialect 名（本例 "postgres"）
// 在 database/sql 中查找驱动，故此处补注。同样注册 "pgx" 名方便直接使用。
func init() {
	sql.Register(dialect.Postgres, &sqlpgx.Driver{})
}

type ServiceContext struct {
	Config config.Config

	// Ent 是数据库 ORM 客户端（PostgreSQL，gen 为生成代码包）。
	Ent *gen.Client
	// Rds 是 Redis 客户端（验证码存储、缓存）。
	Rds *redis.Redis
	// Token 负责签发/解析访问令牌与刷新令牌。
	Token *token.TokenManager
	// Session 是在线会话注册表（登录/刷新时写入，在线会话列表/踢下线读取）。
	Session *session.Manager
	// Sse 是按用户组织的 SSE 事件 Hub（站内信实时通知推送用）。
	Sse *ssehub.Hub
}

func NewServiceContext(c config.Config) *ServiceContext {
	ctx := context.Background()

	entClient, err := newEntClient(ctx, c)
	if err != nil {
		panic(fmt.Sprintf("new ent client failed: %v", err))
	}

	rds := redis.MustNewRedis(c.Redis)

	tm := token.NewTokenManager(
		c.Auth.AccessSecret,
		time.Duration(c.Auth.AccessExpire)*time.Second,
		30*24*time.Hour,
	)

	return &ServiceContext{
		Config:  c,
		Ent:     entClient,
		Rds:     rds,
		Token:   tm,
		Session: session.NewManager(rds),
		Sse:     ssehub.New(),
	}
}

// newEntClient 基于配置的 DataSource 创建 ent 客户端并自动建表。
func newEntClient(ctx context.Context, c config.Config) (*gen.Client, error) {
	drv, err := sqlDriver.Open(c.Mysql.DriverName, c.Mysql.DataSource)
	if err != nil {
		return nil, fmt.Errorf("open %s driver failed: %w", c.Mysql.DriverName, err)
	}

	client := gen.NewClient(gen.Driver(drv))
	if err := client.Schema.Create(ctx, migrate.WithForeignKeys(true)); err != nil {
		client.Close()
		return nil, fmt.Errorf("auto migrate schema failed: %w", err)
	}

	return client, nil
}

// ===== 验证码存储（Redis）=====

const captchaKeyPrefix = "gowind:captcha:"
const captchaTTL = 5 * time.Minute

func captchaKey(id string) string { return captchaKeyPrefix + id }

// CaptchaSave 保存验证码答案。
func (s *ServiceContext) CaptchaSave(id, answer string) error {
	return s.Rds.SetexCtx(context.Background(), captchaKey(id), strings.ToLower(strings.TrimSpace(answer)), int(captchaTTL.Seconds()))
}

// CaptchaVerify 校验验证码并删除（单次有效）。id 为空时返回 false。
func (s *ServiceContext) CaptchaVerify(id, answer string) bool {
	if id == "" || answer == "" {
		return false
	}
	key := captchaKey(id)
	stored, err := s.Rds.GetCtx(context.Background(), key)
	if err != nil || stored == "" {
		return false
	}
	s.Rds.DelCtx(context.Background(), key)
	return strings.EqualFold(strings.TrimSpace(stored), strings.TrimSpace(answer))
}
