// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/api"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ApiDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiDeleteLogic {
	return &ApiDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ApiDeleteLogic) ApiDelete(req *types.ApiDeleteReq) error {
	existing, err := l.svcCtx.Ent.Api.Query().
		Where(api.IDEQ(uint32(req.Id)), api.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("api not found")
		}
		logx.WithContext(l.ctx).Errorf("get api for delete failed: %v", err)
		return xerr.ServerErrorMsg("get api for delete failed")
	}

	if err := l.svcCtx.Ent.Api.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete api failed: %v", err)
		return xerr.ServerErrorMsg("soft delete api failed")
	}

	return nil
}