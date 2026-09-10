// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptCountLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptCountLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptCountLogic {
	return &ScriptCountLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptCountLogic) ScriptCount(req *types.PageRequest) (resp *types.CountScriptsResponse, err error) {
	q := l.svcCtx.Ent.Script.Query().Where(script.DeletedAtIsNil())

	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			script.Or(
				script.NameContainsFold(k),
				script.HookPointContainsFold(k),
			),
		)
	}

	count, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count scripts failed: %v", err)
		return nil, err
	}

	return &types.CountScriptsResponse{Count: int64(count)}, nil
}
