// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user_profile

import (
	"context"

	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	userlogic "go-wind-admin/backendz/internal/logic/user"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserProfileGetUserLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserProfileGetUserLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserProfileGetUserLogic {
	return &UserProfileGetUserLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserProfileGetUserLogic) UserProfileGetUser() (resp *types.User, err error) {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return nil, xerr.UnauthorizedMsg("unauthorized")
	}
	e, err := l.svcCtx.Ent.User.Query().
		Where(genuser.IDEQ(c.UserID), genuser.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("get current user failed: %v", err)
		return nil, xerr.NotFoundMsg("user not found")
	}
	return userlogic.UserToType(l.ctx, l.svcCtx.Ent, e)
}
