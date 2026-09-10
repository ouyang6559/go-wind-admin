// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type NotificationChannelListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewNotificationChannelListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *NotificationChannelListLogic {
	return &NotificationChannelListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *NotificationChannelListLogic) NotificationChannelList(req *types.PageRequest) (resp *types.ListNotificationChannelResponse, err error) {
	// todo: add your logic here and delete this line

	return
}
