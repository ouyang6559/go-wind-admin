// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/notificationchannel"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type NotificationChannelUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewNotificationChannelUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *NotificationChannelUpdateLogic {
	return &NotificationChannelUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *NotificationChannelUpdateLogic) NotificationChannelUpdate(req *types.UpdateNotificationChannelRequest) error {
	d := req.Data

	existing, err := l.svcCtx.Ent.NotificationChannel.Query().
		Where(notificationchannel.IDEQ(uint32(req.Id)), notificationchannel.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("notification channel not found")
		}
		logx.WithContext(l.ctx).Errorf("get notification channel for update failed: %v", err)
		return xerr.ServerErrorMsg("get notification channel failed")
	}

	// 改名时校验唯一性
	if n := strings.TrimSpace(d.Name); n != "" && !strings.EqualFold(n, existing.Name) {
		exists, eerr := l.svcCtx.Ent.NotificationChannel.Query().
			Where(notificationchannel.NameEQ(n), notificationchannel.DeletedAtIsNil(), notificationchannel.IDNEQ(existing.ID)).
			Exist(l.ctx)
		if eerr != nil {
			logx.WithContext(l.ctx).Errorf("check notification channel name during update failed: %v", eerr)
			return xerr.ServerErrorMsg("check channel name failed")
		}
		if exists {
			return xerr.BadRequestMsg("channel name already exists")
		}
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.NotificationChannel.UpdateOneID(existing.ID).
		SetNillableUpdatedBy(&operatorID).
		SetUpdatedAt(time.Now())

	if n := strings.TrimSpace(d.Name); n != "" {
		upd.SetNillableName(strPtr(n))
	}
	if d.SmtpHost != "" {
		upd.SetNillableSMTPHost(strPtr(d.SmtpHost))
	}
	if d.SmtpPort != 0 {
		upd.SetNillableSMTPPort(uint32Ptr(d.SmtpPort))
	}
	if d.SmtpUsername != "" {
		upd.SetNillableSMTPUsername(strPtr(d.SmtpUsername))
	}
	if d.SmtpFrom != "" {
		upd.SetNillableSMTPFrom(strPtr(d.SmtpFrom))
	}
	if d.SmtpHost != "" || d.SmtpPort != 0 || d.SmtpUsername != "" || d.SmtpFrom != "" {
		if t := tlsFromInt(d.SmtpTls); t != nil {
			upd.SetNillableSMTPTLS(t)
		}
	}
	if strings.TrimSpace(d.Remark) != "" {
		upd.SetNillableRemark(strPtr(strings.TrimSpace(d.Remark)))
	}
	// enabled 显式更新：只在前台传了 enabled 时覆盖状态（bool 无法区分未传，按 d.Enabled 整体覆盖）
	if d.Enabled != enabledFromStatus(existing.Status) {
		s := statusFromEnabled(d.Enabled)
		upd.SetNillableStatus(&s)
	}
	// 明文密码；留空表示不修改已存密码
	if p := strings.TrimSpace(req.Password); p != "" {
		upd.SetNillableSMTPPassword(strPtr(p))
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update notification channel [%d] failed: %v", req.Id, uerr)
		return xerr.ServerErrorMsg("update notification channel failed")
	}

	return nil
}
