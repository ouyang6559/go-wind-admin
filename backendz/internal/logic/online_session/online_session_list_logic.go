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

type OnlineSessionListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOnlineSessionListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OnlineSessionListLogic {
	return &OnlineSessionListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// OnlineSessionList 返回全部在线会话，支持按用户名/IP 关键词过滤，登录时间倒序 + 内存分页。
func (l *OnlineSessionListLogic) OnlineSessionList(req *types.ListOnlineSessionRequest) (resp *types.ListOnlineSessionResponse, err error) {
	entries, err := l.svcCtx.Session.List(l.ctx, "*")
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list online sessions failed: %v", err)
		return nil, xerr.ServerErrorMsg("list online sessions failed")
	}

	keyword := strings.ToLower(strings.TrimSpace(req.Keyword))
	items := make([]types.OnlineSession, 0, len(entries))
	for _, m := range entries {
		if keyword != "" &&
			!strings.Contains(strings.ToLower(m.Username), keyword) &&
			!strings.Contains(strings.ToLower(m.IP), keyword) {
			continue
		}
		items = append(items, toType(m, false))
	}

	total := uint64(len(items))

	// 内存分页：page 从 1 开始，缺省每页 20 条
	page := req.Page
	if page < 1 {
		page = 1
	}
	pageSize := req.PageSize
	if pageSize <= 0 {
		pageSize = 20
	}
	start := (page - 1) * pageSize
	var paged []types.OnlineSession
	if start < int64(len(items)) {
		end := start + pageSize
		if end > int64(len(items)) {
			end = int64(len(items))
		}
		paged = items[start:end]
	} else {
		paged = []types.OnlineSession{}
	}

	return &types.ListOnlineSessionResponse{
		Items: paged,
		Total: total,
	}, nil
}
