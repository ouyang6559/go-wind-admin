package data

import (
	"context"
	"time"

	"github.com/tx7do/kratos-bootstrap/bootstrap"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"

	entCrud "github.com/tx7do/go-crud/entgo"

	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/app/admin/service/internal/data/ent/scriptlog"
)

// ScriptLogRepo 脚本执行日志仓储：仅写入与简单查询，不走 proto/管理面。
// 高频写入路径，方法保持最小面。
type ScriptLogRepo struct {
	entClient *entCrud.EntClient[*ent.Client]
	log       *bLogger.Helper
}

func NewScriptLogRepo(ctx *bootstrap.Context, entClient *entCrud.EntClient[*ent.Client]) *ScriptLogRepo {
	return &ScriptLogRepo{
		entClient: entClient,
		log:       ctx.NewLoggerHelper("script-log/repo/admin-service"),
	}
}

// ScriptLogRecord 一次脚本执行的记录。
type ScriptLogRecord struct {
	ScriptID   uint32
	ScriptName string
	Language   string
	Trigger    string // hook / task / test_run / manual
	HookPoint  string
	Version    uint32
	Success    bool
	DurationMS int64
	Error      string
}

// Record 落一条执行日志。失败只记运维日志，不影响业务调用方。
func (r *ScriptLogRepo) Record(ctx context.Context, rec ScriptLogRecord) {
	err := r.entClient.Client().ScriptLog.Create().
		SetNillableScriptID(nonZero(rec.ScriptID)).
		SetNillableScriptName(nonEmptyStr(rec.ScriptName)).
		SetNillableLanguage(nonEmptyStr(rec.Language)).
		SetNillableTriggerType(nonEmptyStr(rec.Trigger)).
		SetNillableHookPoint(nonEmptyStr(rec.HookPoint)).
		SetNillableVersion(nonZero(rec.Version)).
		SetSuccess(rec.Success).
		SetDurationMs(rec.DurationMS).
		SetNillableError(nonEmptyStr(rec.Error)).
		SetCreatedAt(time.Now()).
		Exec(ctx)
	if err != nil {
		r.log.Errorf(ctx, "record script log failed: %s", err.Error())
	}
}

// PurgeBefore 删除指定时间之前的日志（滚动清理），返回删除行数。
func (r *ScriptLogRepo) PurgeBefore(ctx context.Context, before time.Time) (int, error) {
	deleted, err := r.entClient.Client().ScriptLog.Delete().
		Where(scriptlog.CreatedAtLT(before)).
		Exec(ctx)
	if err != nil {
		return 0, err
	}
	return deleted, nil
}

func nonZero(v uint32) *uint32 {
	if v == 0 {
		return nil
	}
	return &v
}

func nonEmptyStr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}
