package service

import (
	"context"
	"sort"
	"strings"
	"time"

	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"github.com/tx7do/go-utils/trans"
	"github.com/tx7do/kratos-bootstrap/bootstrap"
	"google.golang.org/protobuf/types/known/timestamppb"

	adminV1 "go-wind-admin/api/gen/go/admin/service/v1"
	onlineSessionV1 "go-wind-admin/api/gen/go/online_session/service/v1"

	"go-wind-admin/app/admin/service/internal/data"
)

// OnlineSessionService 提供在线会话视图（基于 Redis 令牌对元数据）与强制下线能力。
// 会话记录在令牌签发时写入、随 refresh token 过期或吊销时删除，
// 本服务只做读取与转发吊销，不持有独立存储。
type OnlineSessionService struct {
	adminV1.OnlineSessionServiceHTTPServer

	log           *bLogger.Helper
	authenticator *data.Authenticator
}

func NewOnlineSessionService(
	ctx *bootstrap.Context,
	authenticator *data.Authenticator,
) *OnlineSessionService {
	return &OnlineSessionService{
		log:           ctx.NewLoggerHelper("online-session/service/admin-service"),
		authenticator: authenticator,
	}
}

// ListOnlineSession 返回当前在线会话列表，支持按用户名/IP 关键词过滤，
// 按登录时间倒序排序后在内存中分页。
func (s *OnlineSessionService) ListOnlineSession(ctx context.Context, req *onlineSessionV1.ListOnlineSessionRequest) (*onlineSessionV1.ListOnlineSessionResponse, error) {
	entries, err := s.authenticator.ListSessionEntries(ctx)
	if err != nil {
		s.log.Errorf(ctx, "list session entries failed: %v", err)
		return nil, err
	}

	keyword := strings.ToLower(strings.TrimSpace(req.GetKeyword()))

	items := make([]*onlineSessionV1.OnlineSession, 0, len(entries))
	for _, entry := range entries {
		meta := entry.Meta
		if meta == nil {
			continue
		}
		if keyword != "" &&
			!strings.Contains(strings.ToLower(meta.Username), keyword) &&
			!strings.Contains(strings.ToLower(meta.Ip), keyword) {
			continue
		}

		items = append(items, &onlineSessionV1.OnlineSession{
			Jti:        trans.Ptr(entry.Jti),
			UserId:     trans.Ptr(entry.UserId),
			Username:   trans.Ptr(meta.Username),
			TenantId:   trans.Ptr(meta.TenantId),
			ClientType: trans.Ptr(entry.ClientType),
			IpAddress:  trans.Ptr(meta.Ip),
			UserAgent:  trans.Ptr(meta.UserAgent),
			DeviceId:   trans.Ptr(meta.DeviceId),
			LoginAt:    timestamppb.New(time.Unix(meta.LoginAt, 0)),
		})
	}

	// 登录时间倒序：最新登录的会话排前面
	sort.Slice(items, func(i, j int) bool {
		return items[i].GetLoginAt().AsTime().After(items[j].GetLoginAt().AsTime())
	})

	total := uint64(len(items))

	// 内存分页：page 从 1 开始，缺省返回前 20 条
	page := max(int(req.GetPage()), 1)
	pageSize := int(req.GetPageSize())
	if pageSize == 0 {
		pageSize = 20
	}
	start := (page - 1) * pageSize
	if start >= len(items) {
		items = nil
	} else {
		end := start + pageSize
		if end > len(items) {
			end = len(items)
		}
		items = items[start:end]
	}

	return &onlineSessionV1.ListOnlineSessionResponse{
		Items: items,
		Total: total,
	}, nil
}

// ForceLogoutSession 强制下线指定会话：吊销其访问令牌、刷新令牌与会话元数据。
func (s *OnlineSessionService) ForceLogoutSession(ctx context.Context, req *onlineSessionV1.ForceLogoutSessionRequest) (*onlineSessionV1.ForceLogoutSessionResponse, error) {
	userId := req.GetUserId()
	jti := req.GetJti()
	if userId == 0 || jti == "" {
		return nil, adminV1.ErrorBadRequest("user id and jti are required")
	}

	if err := s.authenticator.RevokeTokenByJti(ctx, trans.Ptr(req.GetClientType()), userId, jti); err != nil {
		s.log.Errorf(ctx, "force logout session failed for user [%d] jti [%s]: %v", userId, jti, err)
		return nil, err
	}

	s.log.Infof(ctx, "session [%s] of user [%d] force logged out", jti, userId)
	return &onlineSessionV1.ForceLogoutSessionResponse{}, nil
}
