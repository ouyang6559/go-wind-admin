package mfa

import (
	"context"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MfaListEnrolledMethodsLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMfaListEnrolledMethodsLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MfaListEnrolledMethodsLogic {
	return &MfaListEnrolledMethodsLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MfaListEnrolledMethodsLogic) MfaListEnrolledMethods(req *types.MfaListEnrolledMethodsReq) (resp *types.ListEnrolledMethodsResponse, err error) {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return nil, xerr.UnauthorizedMsg("unauthorized")
	}

	factors, ferr := listFactorsByUser(l.ctx, l.svcCtx.Ent, c.TenantID, c.UserID)
	if ferr != nil {
		logx.WithContext(l.ctx).Errorf("list mfa factors failed: %v", ferr)
		return nil, xerr.ServerErrorMsg("list mfa factors failed")
	}
	items := make([]types.EnrolledMethod, 0, len(factors))
	for _, f := range factors {
		items = append(items, toEnrolledMethod(f))
	}
	return &types.ListEnrolledMethodsResponse{Items: items}, nil
}