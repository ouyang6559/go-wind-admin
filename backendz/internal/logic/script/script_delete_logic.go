// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptDeleteLogic {
	return &ScriptDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptDeleteLogic) ScriptDelete(req *types.DeleteScriptReq) error {
	if len(req.Ids) == 0 {
		return xerr.BadRequestMsg("script ids required")
	}

	ids := make([]uint32, 0, len(req.Ids))
	for _, id := range req.Ids {
		if id > 0 {
			ids = append(ids, uint32(id))
		}
	}
	if len(ids) == 0 {
		return xerr.BadRequestMsg("script ids required")
	}

	n, err := l.svcCtx.Ent.Script.Update().
		Where(script.IDIn(ids...), script.DeletedAtIsNil()).
		SetDeletedAt(time.Now()).
		Save(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete scripts failed: %v", err)
		return xerr.ServerErrorMsg("soft delete scripts failed")
	}
	logx.WithContext(l.ctx).Infof("soft delete %d scripts", n)

	return nil
}
