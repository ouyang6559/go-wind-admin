// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package position

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/position"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PositionDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPositionDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PositionDeleteLogic {
	return &PositionDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PositionDeleteLogic) PositionDelete(req *types.PositionDeleteReq) error {
	existing, err := l.svcCtx.Ent.Position.Query().
		Where(position.IDEQ(uint32(req.Id)), position.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("position not found")
		}
		logx.WithContext(l.ctx).Errorf("get position for delete failed: %v", err)
		return xerr.ServerErrorMsg("get position for delete failed")
	}

	if err := l.svcCtx.Ent.Position.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete position failed: %v", err)
		return xerr.ServerErrorMsg("soft delete position failed")
	}

	return nil
}
