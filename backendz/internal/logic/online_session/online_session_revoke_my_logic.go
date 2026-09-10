// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package online_session

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type OnlineSessionRevokeMyLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOnlineSessionRevokeMyLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OnlineSessionRevokeMyLogic {
	return &OnlineSessionRevokeMyLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// OnlineSessionRevokeMy 当前用户强制下线自己的指定会话。
// 会话键由 (当前 uid, jti) 唯一确定，天然限定只能操作本人会话；
// 会话不存在（已过期/已下线）时明确报错而非静默成功。
func (l *OnlineSessionRevokeMyLogic) OnlineSessionRevokeMy(req *types.RevokeMyOnlineSessionRequest) error {
	claims, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}

	jti := strings.TrimSpace(req.Jti)
	if jti == "" {
		return xerr.BadRequestMsg("jti is required")
	}

	exists, err := l.svcCtx.Session.Exists(l.ctx, claims.UserID, jti)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("check my session failed for user [%d] jti [%s]: %v", claims.UserID, jti, err)
		return xerr.ServerErrorMsg("check session failed")
	}
	if !exists {
		return xerr.NotFoundMsg("session not found or already offline")
	}

	if err := l.svcCtx.Session.Revoke(l.ctx, claims.UserID, jti); err != nil {
		logx.WithContext(l.ctx).Errorf("revoke my session failed for user [%d] jti [%s]: %v", claims.UserID, jti, err)
		return xerr.ServerErrorMsg("revoke session failed")
	}

	logx.WithContext(l.ctx).Infof("user [%d] revoked own session [%s]", claims.UserID, jti)
	return nil
}
