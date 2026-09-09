// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package mfa

import (
	"context"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MfaGetMFAStatusLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMfaGetMFAStatusLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MfaGetMFAStatusLogic {
	return &MfaGetMFAStatusLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MfaGetMFAStatusLogic) MfaGetMFAStatus(req *types.MfaGetMFAStatusReq) (resp *types.GetMFAStatusResponse, err error) {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return nil, xerr.UnauthorizedMsg("unauthorized")
	}

	hasTotp, err := hasEnabledTotp(l.ctx, l.svcCtx.Ent, c.TenantID, c.UserID)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("check mfa status failed: %v", err)
		return nil, xerr.ServerErrorMsg("check mfa status failed")
	}

	resp = &types.GetMFAStatusResponse{
		Enabled:     hasTotp,
		Enforcement: "MFA_NOT_REQUIRED",
	}
	if hasTotp {
		resp.Enforcement = "MFA_REQUIRED"
		factors, err := listFactorsByUser(l.ctx, l.svcCtx.Ent, c.TenantID, c.UserID)
		if err != nil {
			logx.WithContext(l.ctx).Errorf("list mfa factors failed: %v", err)
			return nil, xerr.ServerErrorMsg("list mfa factors failed")
		}
		enrolled := make([]types.EnrolledMethod, 0, len(factors))
		for _, f := range factors {
			enrolled = append(enrolled, toEnrolledMethod(f))
		}
		resp.Enrolled = enrolled
	}
	return resp, nil
}