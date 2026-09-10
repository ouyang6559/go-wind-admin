// Code scaffolded by goctl. Safe to edit.

package online_session

import (
	"time"

	"go-wind-admin/backendz/internal/pkg/session"
	"go-wind-admin/backendz/internal/types"
)

// toType 将会话注册表条目转换为 API 展示模型。
// current 标记该会话是否为当前请求所属会话（仅 my-sessions 场景传入 true）。
func toType(m session.Meta, current bool) types.OnlineSession {
	return types.OnlineSession{
		Current:    current,
		Jti:        m.JTI,
		UserId:     m.UID,
		Username:   m.Username,
		TenantId:   m.TenantID,
		ClientType: m.ClientType,
		IpAddress:  m.IP,
		UserAgent:  m.UserAgent,
		DeviceId:   m.DeviceID,
		LoginAt:    m.LoginAt.UTC().Format(time.RFC3339),
	}
}
