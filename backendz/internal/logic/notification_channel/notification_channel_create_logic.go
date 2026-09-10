// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

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
	// todo: add your logic here and delete this line

	return nil
}
