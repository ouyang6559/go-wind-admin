// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/api"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ApiUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiUpdateLogic {
	return &ApiUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ApiUpdateLogic) ApiUpdate(req *types.UpdateApiRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Api.Query().
		Where(api.IDEQ(uint32(req.Id)), api.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("api not found")
		}
		logx.WithContext(l.ctx).Errorf("get api for update failed: %v", err)
		return xerr.ServerErrorMsg("get api for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.Api.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Operation != "" {
		upd.SetOperation(d.Operation)
	}
	if d.Path != "" {
		upd.SetPath(d.Path)
	}
	if d.Method != "" {
		upd.SetMethod(d.Method)
	}
	if d.Module != "" {
		upd.SetModule(d.Module)
	}
	if d.ModuleDescription != "" {
		upd.SetModuleDescription(d.ModuleDescription)
	}
	if d.Description != "" {
		upd.SetDescription(d.Description)
	}
	if d.Status != "" {
		upd.SetStatus(api.Status(d.Status))
	}
	if d.BusinessModule != "" {
		upd.SetBusinessModule(api.BusinessModule(d.BusinessModule))
	}
	if d.Scope != "" {
		upd.SetScope(api.Scope(d.Scope))
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update api failed: %v", uerr)
		return xerr.ServerErrorMsg("update api failed")
	}

	return nil
}