// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"
	"strconv"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/notificationchannel"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/mailer"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type NotificationChannelSendTestEmailLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewNotificationChannelSendTestEmailLogic(ctx context.Context, svcCtx *svc.ServiceContext) *NotificationChannelSendTestEmailLogic {
	return &NotificationChannelSendTestEmailLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// SendTestEmail 向指定收件人发送测试邮件，验证渠道配置是否可用。
// 渠道未启用时拒绝发送，避免误以为配置可用。
func (l *NotificationChannelSendTestEmailLogic) NotificationChannelSendTestEmail(req *types.SendTestEmailRequest) error {
	if req.Id <= 0 {
		return xerr.BadRequestMsg("notification channel id required")
	}
	if strings.TrimSpace(req.Recipient) == "" {
		return xerr.BadRequestMsg("recipient required")
	}

	row, err := l.svcCtx.Ent.NotificationChannel.Query().
		Where(notificationchannel.IDEQ(uint32(req.Id)), notificationchannel.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("notification channel not found")
		}
		logx.WithContext(l.ctx).Errorf("get notification channel [%d] failed: %v", req.Id, err)
		return xerr.ServerErrorMsg("get notification channel failed")
	}

	// 测试邮件仅支持 EMAIL 渠道
	if row.Type != notificationchannel.TypeEmail {
		return xerr.BadRequestMsg("test email is only available for EMAIL channels")
	}
	// 未启用时拒绝发送
	if !enabledFromStatus(row.Status) {
		return xerr.BadRequestMsg("notification channel is disabled")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	// SMTP 密码明文存储（与 create 保持一致），此处原样读出
	password := ""
	if row.SMTPPassword != nil {
		password = *row.SMTPPassword
	}

	subject := "GoWind Admin 通知渠道测试邮件"
	body := "这是一封来自 GoWind Admin 的测试邮件。\n" +
		"如果您收到了它，说明渠道 [" + strconv.FormatUint(uint64(req.Id), 10) + "] 配置可用。\n" +
		"操作人用户 ID: " + strconv.FormatUint(uint64(operatorID), 10) + "\n"

	err = mailer.SendMail(mailer.SmtpConfig{
		Host:     std.Str(row.SMTPHost),
		Port:     uint32Val(row.SMTPPort),
		Username: std.Str(row.SMTPUsername),
		Password: password,
		From:     std.Str(row.SMTPFrom),
		TlsMode:  tlsToString(row.SMTPTLS),
	}, []string{strings.TrimSpace(req.Recipient)}, subject, body)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("send test email via channel [%d] to [%s] failed: %v", req.Id, req.Recipient, err)
		return xerr.BadRequestMsg("send test email failed: " + err.Error())
	}

	logx.WithContext(l.ctx).Infof("test email sent via channel [%d] to [%s] by operator [%d]", req.Id, req.Recipient, operatorID)
	return nil
}

// tlsToString ent smtp_tls 指针 -> "NONE"/"START_TLS"/"SSL"；nil 返回空串。
func tlsToString(t *notificationchannel.SMTPTLS) string {
	if t == nil {
		return ""
	}
	return string(*t)
}
