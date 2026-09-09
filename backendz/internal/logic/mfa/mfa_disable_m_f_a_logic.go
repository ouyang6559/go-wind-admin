package mfa

import (
	"context"
	"strconv"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/usermfafactor"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MfaDisableMFALogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMfaDisableMFALogic(ctx context.Context, svcCtx *svc.ServiceContext) *MfaDisableMFALogic {
	return &MfaDisableMFALogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MfaDisableMFALogic) MfaDisableMFA(req *types.DisableMFARequest) error {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}

	// 目标用户：默认当前用户，管理员可传 userId。
	targetUser := c.UserID
	if req.UserId > 0 {
		targetUser = uint32(req.UserId)
	}

	q := l.svcCtx.Ent.UserMfaFactor.Update().
		Where(usermfafactor.TenantID(c.TenantID), usermfafactor.UserID(targetUser))

	if id := req.CredentialId; id != "" {
		if fid, perr := strconv.ParseUint(id, 10, 32); perr == nil {
			q.Where(usermfafactor.ID(uint32(fid)))
		} else {
			return xerr.NotFoundMsg("invalid credential")
		}
	} else if m := req.Method; m != "" {
		q.Where(usermfafactor.MethodEQ(usermfafactor.Method(m)))
	}

	n, uerr := q.
		SetStatus(usermfafactor.StatusDisabled).
		SetDeletedAt(time.Now()).
		Save(l.ctx)
	if uerr != nil {
		logx.WithContext(l.ctx).Errorf("disable mfa failed: %v", uerr)
		return xerr.ServerErrorMsg("disable mfa failed")
	}
	if n == 0 {
		return xerr.NotFoundMsg("mfa factor not found")
	}
	return nil
}