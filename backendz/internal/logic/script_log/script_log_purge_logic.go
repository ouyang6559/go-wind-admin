// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script_log

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/scriptlog"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptLogPurgeLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptLogPurgeLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptLogPurgeLogic {
	return &ScriptLogPurgeLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptLogPurgeLogic) ScriptLogPurge(req *types.PurgeScriptLogsRequest) (resp *types.PurgeScriptLogsResponse, err error) {
	threshold := strings.TrimSpace(req.Before)
	if threshold == "" {
		return nil, xerr.BadRequestMsg("before required")
	}

	before, perr := time.Parse(time.RFC3339, threshold)
	if perr != nil {
		return nil, xerr.BadRequestMsg("invalid before time")
	}

	n, derr := l.svcCtx.Ent.ScriptLog.Update().
		Where(scriptlog.DeletedAtIsNil(), scriptlog.CreatedAtLT(before)).
		SetDeletedAt(time.Now()).
		Save(l.ctx)
	if derr != nil {
		logx.WithContext(l.ctx).Errorf("purge script logs failed: %v", derr)
		return nil, xerr.ServerErrorMsg("purge script logs failed")
	}

	return &types.PurgeScriptLogsResponse{Deleted: int64(n)}, nil
}
