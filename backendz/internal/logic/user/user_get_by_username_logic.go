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

type UserGetByUsernameLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserGetByUsernameLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserGetByUsernameLogic {
	return &UserGetByUsernameLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserGetByUsernameLogic) UserGetByUsername(req *types.UserGetByUsernameReq) (resp *types.User, err error) {
	e, err := l.svcCtx.Ent.User.Query().
		Where(genuser.UsernameEQ(req.Username), genuser.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("user not found")
		}
		logx.WithContext(l.ctx).Errorf("get user by username failed: %v", err)
		return nil, err
	}
	return UserToType(l.ctx, l.svcCtx.Ent, e)
}
