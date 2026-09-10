// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

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

func (l *NotificationChannelSendTestEmailLogic) NotificationChannelSendTestEmail(req *types.SendTestEmailRequest) error {
	// todo: add your logic here and delete this line

	return nil
}
