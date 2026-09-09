// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package position

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/position"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PositionGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPositionGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PositionGetLogic {
	return &PositionGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PositionGetLogic) PositionGet(req *types.PositionGetReq) (resp *types.Position, err error) {
	var q = l.svcCtx.Ent.Position.Query().Where(position.DeletedAtIsNil())
	if req.Id > 0 {
		q = q.Where(position.IDEQ(uint32(req.Id)))
	} else if req.Code != "" {
		q = q.Where(position.CodeEQ(req.Code))
	} else if req.Name != "" {
		q = q.Where(position.NameEQ(req.Name))
	} else {
		return nil, xerr.BadRequestMsg("id or code required")
	}

	e, err := q.Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("position not found")
		}
		logx.WithContext(l.ctx).Errorf("get position failed: %v", err)
		return nil, err
	}

	return toType(l.ctx, l.svcCtx.Ent, e), nil
}
