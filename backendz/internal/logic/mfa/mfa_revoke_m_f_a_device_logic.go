package mfa

import (
	"context"
	"strconv"

	"go-wind-admin/backendz/internal/ent/gen/usermfafactor"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MfaRevokeMFADeviceLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMfaRevokeMFADeviceLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MfaRevokeMFADeviceLogic {
	return &MfaRevokeMFADeviceLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MfaRevokeMFADeviceLogic) MfaRevokeMFADevice(req *types.MfaRevokeMFADeviceReq) error {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}

	fid, perr := strconv.ParseUint(req.CredentialId, 10, 32)
	if perr != nil {
		return xerr.BadRequestMsg("invalid credentialId")
	}

	n, uerr := l.svcCtx.Ent.UserMfaFactor.Update().
		Where(
			usermfafactor.ID(uint32(fid)),
			usermfafactor.TenantID(c.TenantID),
			usermfafactor.UserID(c.UserID),
		).
		SetStatus(usermfafactor.StatusDisabled).
		SetSecretHash("").
		Save(l.ctx)
	if uerr != nil {
		logx.WithContext(l.ctx).Errorf("revoke mfa device failed: %v", uerr)
		return xerr.ServerErrorMsg("revoke mfa device failed")
	}
	if n == 0 {
		return xerr.NotFoundMsg("mfa device not found")
	}
	return nil
}