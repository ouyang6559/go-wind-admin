// 通知渠道 logic 层的 ent<->types 转换与枚举映射辅助。
// 与旧 backend（go-kratos）语义保持一致：
//   - ChannelType：0=EMAIL 1=WEBHOOK（对应 ent enum "EMAIL"/"WEBHOOK"）
//   - SmtpTls：0=NONE 1=START_TLS 2=SSL（对应 ent enum "NONE"/"START_TLS"/"SSL"）
//   - Enabled：ent Status ON/OFF
package notification_channel

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/notificationchannel"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转为 types.NotificationChannel（不含 SMTP 密码，仅 HasPassword 标识）。
func toType(e *gen.NotificationChannel) *types.NotificationChannel {
	return &types.NotificationChannel{
		Id:           int64(e.ID),
		Name:         e.Name,
		ChannelType:  channelTypeToInt(e.Type),
		SmtpHost:     std.Str(e.SMTPHost),
		SmtpPort:     std.Int64(e.SMTPPort),
		SmtpUsername: std.Str(e.SMTPUsername),
		HasPassword:  e.SMTPPassword != nil,
		SmtpFrom:     std.Str(e.SMTPFrom),
		SmtpTls:      tlsToInt(e.SMTPTLS),
		Enabled:      enabledFromStatus(e.Status),
		Remark:       std.Str(e.Remark),
		CreatedBy:    std.Int64(e.CreatedBy),
		UpdatedBy:    std.Int64(e.UpdatedBy),
		DeletedBy:    std.Int64(e.DeletedBy),
		CreatedAt:    std.TimeStr(e.CreatedAt),
		UpdatedAt:    std.TimeStr(e.UpdatedAt),
		DeletedAt:    std.TimeStr(e.DeletedAt),
	}
}

// strPtr 空串返回 nil，否则返回指针。
func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

// uint32Ptr 将 int64 转为 *uint32；0 返回 nil。
func uint32Ptr(v int64) *uint32 {
	if v == 0 {
		return nil
	}
	p := uint32(v)
	return &p
}

// uint32Val 解引用 *uint32；nil 返回 0。
func uint32Val(v *uint32) uint32 {
	if v == nil {
		return 0
	}
	return *v
}

// channelTypeFromInt 0=EMAIL 1=WEBHOOK -> ent type；非法值退回 EMAIL。
func channelTypeFromInt(v int64) *notificationchannel.Type {
	switch v {
	case 1:
		p := notificationchannel.TypeWebhook
		return &p
	default:
		p := notificationchannel.TypeEmail
		return &p
	}
}

// channelTypeToInt ent type -> int64（0=EMAIL 1=WEBHOOK）。
func channelTypeToInt(t notificationchannel.Type) int64 {
	if t == notificationchannel.TypeWebhook {
		return 1
	}
	return 0
}

// tlsFromInt 0=NONE 1=START_TLS 2=SSL -> ent smtp_tls；非法值退回 NONE。
func tlsFromInt(v int64) *notificationchannel.SMTPTLS {
	switch v {
	case 1:
		p := notificationchannel.SMTPTLSStartTls
		return &p
	case 2:
		p := notificationchannel.SMTPTLSSsl
		return &p
	default:
		p := notificationchannel.SMTPTLSNone
		return &p
	}
}

// tlsToInt ent smtp_tls -> int64（0=NONE 1=START_TLS 2=SSL）。
func tlsToInt(t *notificationchannel.SMTPTLS) int64 {
	if t == nil {
		return 0
	}
	switch *t {
	case notificationchannel.SMTPTLSStartTls:
		return 1
	case notificationchannel.SMTPTLSSsl:
		return 2
	default:
		return 0
	}
}

// statusFromEnabled bool -> Status ON/OFF。
func statusFromEnabled(enabled bool) notificationchannel.Status {
	if enabled {
		return notificationchannel.StatusOn
	}
	return notificationchannel.StatusOff
}

// enabledFromStatus Status 指针 -> bool（nil 视为 OFF）。
func enabledFromStatus(s *notificationchannel.Status) bool {
	return s != nil && *s == notificationchannel.StatusOn
}
