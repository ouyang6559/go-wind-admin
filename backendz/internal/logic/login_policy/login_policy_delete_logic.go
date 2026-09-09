// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package login_policy

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/loginpolicy"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LoginPolicyDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLoginPolicyDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LoginPolicyDeleteLogic {
	return &LoginPolicyDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LoginPolicyDeleteLogic) LoginPolicyDelete(req *types.LoginPolicyDeleteReq) error {
	existing, err := l.svcCtx.Ent.LoginPolicy.Query().
		Where(loginpolicy.IDEQ(uint32(req.Id)), loginpolicy.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("login policy not found")
		}
		logx.WithContext(l.ctx).Errorf("get login policy for delete failed: %v", err)
		return xerr.ServerErrorMsg("get login policy for delete failed")
	}

	if err := l.svcCtx.Ent.LoginPolicy.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete login policy failed: %v", err)
		return xerr.ServerErrorMsg("soft delete login policy failed")
	}

	return nil
}
