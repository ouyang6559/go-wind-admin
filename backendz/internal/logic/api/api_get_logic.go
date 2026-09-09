// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/api"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ApiGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiGetLogic {
	return &ApiGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ApiGetLogic) ApiGet(req *types.ApiGetReq) (resp *types.Api, err error) {
	e, err := l.svcCtx.Ent.Api.Query().
		Where(api.IDEQ(uint32(req.Id)), api.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("api not found")
		}
		logx.WithContext(l.ctx).Errorf("get api failed: %v", err)
		return nil, err
	}

	return toType(e), nil
}