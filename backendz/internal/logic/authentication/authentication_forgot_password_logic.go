// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package authentication

import (
	"context"
	"fmt"
	"strings"
	"time"

	"entgo.io/ent/privacy"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/notificationchannel"
	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/pkg/mailer"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type AuthenticationForgotPasswordLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAuthenticationForgotPasswordLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AuthenticationForgotPasswordLogic {
	return &AuthenticationForgotPasswordLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// generateVCode 生成 6 位数字验证码。
// 使用纳秒时间取模：验证码仅用于一次性校验，安全强度由
// 「单次有效 + 10 分钟 TTL + 服务端比对」保证，不依赖密码学随机（与旧 backend 一致）。
func generateVCode() string {
	return fmt.Sprintf("%06d", time.Now().UnixNano()%1000000)
}

// AuthenticationForgotPassword 忘记密码：向 identifier（必须为已绑定的邮箱凭证）发送
// 重置验证码（免鉴权）。用户不存在时同样返回成功，防止通过接口枚举有效邮箱。
func (l *AuthenticationForgotPasswordLogic) AuthenticationForgotPassword(req *types.ForgotPasswordRequest) error {
	identifier := strings.TrimSpace(req.Identifier)
	if identifier == "" {
		return xerr.BadRequestMsg("identifier is required")
	}

	allowCtx := privacy.DecisionContext(l.ctx, privacy.Allow)

	// 仅向已绑定 EMAIL 凭证的标识符发送验证码
	cred, cerr := l.svcCtx.Ent.UserCredential.Query().
		Where(
			usercredential.IdentityTypeEQ(usercredential.IdentityTypeEmail),
			usercredential.IdentifierEQ(identifier),
		).
		Only(allowCtx)
	if cerr != nil || cred.UserID == nil || *cred.UserID == 0 {
		// 反枚举：无论凭证不存在还是其它错误都回成功，仅记录日志
		logx.WithContext(l.ctx).Infof("forgot-password: no EMAIL credential for [%s] (silent ok): %v", identifier, cerr)
		return nil
	}
	userID := *cred.UserID

	code := generateVCode()
	if err := l.svcCtx.VCodeSave(svc.VCodePurposeResetPassword, identifier, code); err != nil {
		logx.WithContext(l.ctx).Errorf("forgot-password: save verification code for [%s] failed: %v", identifier, err)
		return xerr.ServerErrorMsg("save verification code failed")
	}

	// 获取第一个启用的 EMAIL 通知渠道
	ch, chErr := l.svcCtx.Ent.NotificationChannel.Query().
		Where(
			notificationchannel.TypeEQ(notificationchannel.TypeEmail),
			notificationchannel.StatusEQ(notificationchannel.StatusOn),
		).
		Order(notificationchannel.ByID()).
		First(allowCtx)
	if chErr != nil {
		if gen.IsNotFound(chErr) {
			logx.WithContext(l.ctx).Errorf("forgot-password: no enabled email channel configured")
			return xerr.ServerErrorMsg("email channel is not configured")
		}
		logx.WithContext(l.ctx).Errorf("forgot-password: query email channel failed: %v", chErr)
		return xerr.ServerErrorMsg("query email channel failed")
	}

	// 与通知渠道模块约定一致：SMTP 密码明文存储，这里原样读出
	smtpPassword := ""
	if ch.SMTPPassword != nil {
		smtpPassword = *ch.SMTPPassword
	}

	subject := "GoWind Admin 密码重置验证码"
	body := "您的密码重置验证码是：" + code + "\n\n10 分钟内有效。若非本人操作请忽略本邮件。\n"
	if err := mailer.SendMail(mailer.SmtpConfig{
		Host:     std.Str(ch.SMTPHost),
		Port:     uint32ValPtr(ch.SMTPPort),
		Username: std.Str(ch.SMTPUsername),
		Password: smtpPassword,
		From:     std.Str(ch.SMTPFrom),
		TlsMode:  tlsModeStr(ch.SMTPTLS),
	}, []string{identifier}, subject, body); err != nil {
		logx.WithContext(l.ctx).Errorf("forgot-password: send mail to [%s] failed: %v", identifier, err)
		return xerr.ServerErrorMsg("send verification email failed")
	}

	logx.WithContext(l.ctx).Infof("forgot-password: reset code sent to [%s] for user [%d]", identifier, userID)
	return nil
}

// uint32ValPtr 解引用 *uint32；nil 返回 0。
func uint32ValPtr(v *uint32) uint32 {
	if v == nil {
		return 0
	}
	return *v
}

// tlsModeStr ent smtp_tls 指针 -> "NONE"/"START_TLS"/"SSL"；nil 返回空串。
func tlsModeStr(t *notificationchannel.SMTPTLS) string {
	if t == nil {
		return ""
	}
	return string(*t)
}