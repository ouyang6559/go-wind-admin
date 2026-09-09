// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/api"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ApiCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiCreateLogic {
	return &ApiCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ApiCreateLogic) ApiCreate(req *types.CreateApiRequest) error {
	d := req.Data
	if d.Path == "" {
		return xerr.BadRequestMsg("api path required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	status := api.StatusOn
	if d.Status != "" {
		status = api.Status(d.Status)
	}

	// business_module 仅在显式指定时写入（无默认值，留空则保持 NULL）
	scope := api.ScopeAdmin
	if d.Scope != "" {
		scope = api.Scope(d.Scope)
	}
	create := l.svcCtx.Ent.Api.Create().
		SetStatus(status).
		SetNillableOperation(nilStr(d.Operation)).
		SetPath(d.Path).
		SetNillableMethod(nilStr(d.Method)).
		SetNillableModule(nilStr(d.Module)).
		SetNillableModuleDescription(nilStr(d.ModuleDescription)).
		SetNillableDescription(nilStr(d.Description)).
		SetScope(scope).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())
	if d.BusinessModule != "" {
		create = create.SetBusinessModule(api.BusinessModule(d.BusinessModule))
	}

	if _, cerr := create.Save(l.ctx); cerr != nil {
		logx.WithContext(l.ctx).Errorf("create api failed: %v", cerr)
		return xerr.ServerErrorMsg("create api failed")
	}

	return nil
}

func nilStr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}
