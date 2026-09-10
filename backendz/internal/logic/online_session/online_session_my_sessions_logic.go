// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package online_session

import (
	"context"
	"fmt"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type OnlineSessionMySessionsLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOnlineSessionMySessionsLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OnlineSessionMySessionsLogic {
	return &OnlineSessionMySessionsLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// OnlineSessionMySessions 返回当前登录用户的全部在线会话（个人中心自助视图）。
// 用户身份取自认证上下文；不分页（单用户会话数有限），current 标记当前请求所属会话。
func (l *OnlineSessionMySessionsLogic) OnlineSessionMySessions() (resp *types.ListOnlineSessionResponse, err error) {
	claims, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return nil, xerr.UnauthorizedMsg("unauthorized")
	}

	entries, err := l.svcCtx.Session.List(l.ctx, fmt.Sprintf("%d:*", claims.UserID))
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list my sessions failed for user [%d]: %v", claims.UserID, err)
		return nil, xerr.ServerErrorMsg("list my sessions failed")
	}

	items := make([]types.OnlineSession, 0, len(entries))
	for _, m := range entries {
		items = append(items, toType(m, m.JTI == claims.ID))
	}

	return &types.ListOnlineSessionResponse{
		Items: items,
		Total: uint64(len(items)),
	}, nil
}
