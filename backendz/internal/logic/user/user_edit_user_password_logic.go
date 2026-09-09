// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserEditUserPasswordLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserEditUserPasswordLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserEditUserPasswordLogic {
	return &UserEditUserPasswordLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserEditUserPasswordLogic) UserEditUserPassword(req *types.EditUserPasswordRequest) error {
	u, err := l.svcCtx.Ent.User.Query().
		Where(genuser.IDEQ(uint32(req.UserId)), genuser.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("user not found")
		}
		logx.WithContext(l.ctx).Errorf("get user for password edit failed: %v", err)
		return xerr.ServerErrorMsg("get user failed")
	}
	return resetCredentialUsername(l.ctx, l.svcCtx, u.Username, req.NewPassword)
}
