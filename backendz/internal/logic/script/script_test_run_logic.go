// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptTestRunLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptTestRunLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptTestRunLogic {
	return &ScriptTestRunLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptTestRunLogic) ScriptTestRun(req *types.TestRunScriptRequest) (resp *types.TestRunScriptResponse, err error) {
	start := time.Now()

	var name, language, hookPoint, source string
	var version uint32
	var scriptID uint32

	if req.Id > 0 {
		row, qerr := l.svcCtx.Ent.Script.Query().
			Where(script.IDEQ(uint32(req.Id)), script.DeletedAtIsNil()).
			Only(l.ctx)
		if qerr != nil {
			logx.WithContext(l.ctx).Errorf("load script for test_run failed: %v", qerr)
			return nil, xerr.NotFoundMsg("script not found")
		}
		scriptID = row.ID
		name = stdStr(row.Name)
		language = langName(row.Language)
		hookPoint = stdStr(row.HookPoint)
		source = stdStr(row.Source)
		if row.Version != nil {
			version = *row.Version
		}
	} else if d := req.Draft; d.Name != "" {
		// 草稿试运行
		name = strings.TrimSpace(d.Name)
		language = d.Language
		hookPoint = d.HookPoint
		source = d.Source
	} else {
		return nil, xerr.BadRequestMsg("script id or draft required")
	}

	// 说明：本服务不承载脚本引擎（不碰引擎），试运行仅记录一条执行日志作为结果。
	now := time.Now()
	duration := time.Since(start).Milliseconds()

	_, lerr := l.svcCtx.Ent.ScriptLog.Create().
		SetNillableScriptID(&scriptID).
		SetNillableScriptName(strPtr(name)).
		SetNillableLanguage(strPtr(language)).
		SetTriggerType("test_run").
		SetNillableHookPoint(strPtr(hookPoint)).
		SetNillableVersion(uintPtr2(version)).
		SetSuccess(true).
		SetNillableDurationMs(&duration).
		SetCreatedAt(now).
		SetUpdatedAt(now).
		Save(l.ctx)
	if lerr != nil {
		logx.WithContext(l.ctx).Errorf("record test_run log failed: %v", lerr)
	}

	return &types.TestRunScriptResponse{
		Success:    true,
		Context:    map[string]string{"source": source},
		DurationMs: duration,
	}, nil
}

func langName(v *script.Language) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

func uintPtr2(v uint32) *uint32 {
	return &v
}
