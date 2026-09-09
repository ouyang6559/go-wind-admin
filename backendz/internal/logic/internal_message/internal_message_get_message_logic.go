// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessage"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageGetMessageLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageGetMessageLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageGetMessageLogic {
	return &InternalMessageGetMessageLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageGetMessageLogic) InternalMessageGetMessage(req *types.InternalMessageGetMessageReq) (resp *types.InternalMessage, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.InternalMessage.Query().
		Where(internalmessage.IDEQ(uint32(req.Id)), internalmessage.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("internal message not found")
		}
		logx.WithContext(l.ctx).Errorf("get internal message failed: %v", err)
		return nil, err
	}

	return toType(l.ctx, l.svcCtx.Ent, e), nil
}