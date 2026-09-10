// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script_log

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen/scriptlog"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptLogCountLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptLogCountLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptLogCountLogic {
	return &ScriptLogCountLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptLogCountLogic) ScriptLogCount(req *types.PageRequest) (resp *types.CountScriptLogsResponse, err error) {
	q := l.svcCtx.Ent.ScriptLog.Query().Where(scriptlog.DeletedAtIsNil())

	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			scriptlog.Or(
				scriptlog.ScriptNameContainsFold(k),
				scriptlog.HookPointContainsFold(k),
				scriptlog.TriggerTypeContainsFold(k),
			),
		)
	}

	count, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count script logs failed: %v", err)
		return nil, err
	}

	return &types.CountScriptLogsResponse{Count: int64(count)}, nil
}
