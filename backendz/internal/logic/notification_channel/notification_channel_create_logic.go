// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/notificationchannel"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type NotificationChannelCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewNotificationChannelCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *NotificationChannelCreateLogic {
	return &NotificationChannelCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *NotificationChannelCreateLogic) NotificationChannelCreate(req *types.CreateNotificationChannelRequest) error {
	d := req.Data
	name := strings.TrimSpace(d.Name)
	if name == "" {
		return xerr.BadRequestMsg("channel name required")
	}

	// 名称唯一校验
	exists, err := l.svcCtx.Ent.NotificationChannel.Query().
		Where(notificationchannel.NameEQ(name), notificationchannel.DeletedAtIsNil()).
		Exist(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("check notification channel name exists failed: %v", err)
		return xerr.ServerErrorMsg("check channel name failed")
	}
	if exists {
		return xerr.BadRequestMsg("channel name already exists")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	now := time.Now()
	b := l.svcCtx.Ent.NotificationChannel.Create().
		SetName(name).
		SetNillableType(channelTypeFromInt(d.ChannelType)).
		SetNillableSMTPHost(strPtr(d.SmtpHost)).
		SetNillableSMTPPort(uint32Ptr(d.SmtpPort)).
		SetNillableSMTPUsername(strPtr(d.SmtpUsername)).
		SetNillableSMTPFrom(strPtr(d.SmtpFrom)).
		SetNillableSMTPTLS(tlsFromInt(d.SmtpTls)).
		SetStatus(statusFromEnabled(d.Enabled)).
		SetNillableRemark(strPtr(d.Remark)).
		SetNillableCreatedBy(&operatorID).
		SetNillableUpdatedBy(&operatorID).
		SetCreatedAt(now).
		SetUpdatedAt(now)

	// 明文密码落库。旧 backend 经由全局 EncryptIfNeeded（未配置密钥时为 no-op，即明文
	// 存储）；backendz 未引入加密子系统，此处保持一致存明文，下游读取亦原样使用。
	if p := strings.TrimSpace(req.Password); p != "" {
		b.SetNillableSMTPPassword(strPtr(p))
	}

	if _, cerr := b.Save(l.ctx); cerr != nil {
		logx.WithContext(l.ctx).Errorf("create notification channel failed: %v", cerr)
		return xerr.ServerErrorMsg("create notification channel failed")
	}

	return nil
}
