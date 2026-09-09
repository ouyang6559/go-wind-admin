// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user_profile

import (
	"context"

	userlogic "go-wind-admin/backendz/internal/logic/user"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserProfileUpdateUserLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserProfileUpdateUserLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserProfileUpdateUserLogic {
	return &UserProfileUpdateUserLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserProfileUpdateUserLogic) UserProfileUpdateUser(req *types.UpdateUserRequest) error {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}
	upd := l.svcCtx.Ent.User.UpdateOneID(c.UserID).SetUpdatedBy(c.UserID)
	userlogic.ApplyUserPatch(upd, req.Data)
	if _, err := upd.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("update current user failed: %v", err)
		return xerr.ServerErrorMsg("update user failed")
	}
	return nil
}
