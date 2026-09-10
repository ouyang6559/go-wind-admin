// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package online_session

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type OnlineSessionForceLogoutLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOnlineSessionForceLogoutLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OnlineSessionForceLogoutLogic {
	return &OnlineSessionForceLogoutLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// OnlineSessionForceLogout 强制下线指定会话（管理者视角）：吊销其令牌对会话记录。
// 与 my-sessions/revoke 不同，这里是幂等的（目标会话不存在也视为已处理成功）。
func (l *OnlineSessionForceLogoutLogic) OnlineSessionForceLogout(req *types.ForceLogoutSessionRequest) error {
	userID := req.UserId
	jti := strings.TrimSpace(req.Jti)
	if userID == 0 || jti == "" {
		return xerr.BadRequestMsg("user id and jti are required")
	}

	if err := l.svcCtx.Session.Revoke(l.ctx, userID, jti); err != nil {
		logx.WithContext(l.ctx).Errorf("force logout session failed for user [%d] jti [%s]: %v", userID, jti, err)
		return xerr.ServerErrorMsg("force logout session failed")
	}

	logx.WithContext(l.ctx).Infof("session [%s] of user [%d] force logged out", jti, userID)
	return nil
}
