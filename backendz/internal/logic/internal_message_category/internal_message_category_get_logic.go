// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message_category

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessagecategory"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageCategoryGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageCategoryGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageCategoryGetLogic {
	return &InternalMessageCategoryGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageCategoryGetLogic) InternalMessageCategoryGet(req *types.InternalMessageCategoryGetReq) (resp *types.InternalMessageCategory, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.InternalMessageCategory.Query().
		Where(internalmessagecategory.IDEQ(uint32(req.Id)), internalmessagecategory.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("internal message category not found")
		}
		logx.WithContext(l.ctx).Errorf("get internal message category failed: %v", err)
		return nil, err
	}

	return toType(l.ctx, l.svcCtx.Ent, e), nil
}