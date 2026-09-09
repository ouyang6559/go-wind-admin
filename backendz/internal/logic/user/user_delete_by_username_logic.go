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

type UserDeleteByUsernameLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserDeleteByUsernameLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserDeleteByUsernameLogic {
	return &UserDeleteByUsernameLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserDeleteByUsernameLogic) UserDeleteByUsername(req *types.UserDeleteByUsernameReq) error {
	u, err := l.svcCtx.Ent.User.Query().
		Where(genuser.UsernameEQ(req.Username), genuser.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("user not found")
		}
		logx.WithContext(l.ctx).Errorf("get user by username for delete failed: %v", err)
		return xerr.ServerErrorMsg("get user failed")
	}
	return softDeleteUser(l.ctx, l.svcCtx, u)
}
