// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type NotificationChannelDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewNotificationChannelDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *NotificationChannelDeleteLogic {
	return &NotificationChannelDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *NotificationChannelDeleteLogic) NotificationChannelDelete(req *types.DeleteNotificationChannelReq) error {
	// todo: add your logic here and delete this line

	return nil
}
