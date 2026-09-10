// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type NotificationChannelGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewNotificationChannelGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *NotificationChannelGetLogic {
	return &NotificationChannelGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *NotificationChannelGetLogic) NotificationChannelGet(req *types.GetNotificationChannelReq) (resp *types.NotificationChannel, err error) {
	// todo: add your logic here and delete this line

	return
}
