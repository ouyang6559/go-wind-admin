// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user_profile

import (
	"context"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserProfileDeleteAvatarLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserProfileDeleteAvatarLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserProfileDeleteAvatarLogic {
	return &UserProfileDeleteAvatarLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserProfileDeleteAvatarLogic) UserProfileDeleteAvatar() error {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}
	if err := l.svcCtx.Ent.User.UpdateOneID(c.UserID).
		SetAvatar("").
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("clear avatar failed: %v", err)
		return xerr.ServerErrorMsg("clear avatar failed")
	}
	return nil
}
