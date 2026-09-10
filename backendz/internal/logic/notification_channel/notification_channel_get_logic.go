// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/notificationchannel"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

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
	row, err := l.svcCtx.Ent.NotificationChannel.Query().
		Where(notificationchannel.IDEQ(uint32(req.Id)), notificationchannel.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("notification channel not found")
		}
		logx.WithContext(l.ctx).Errorf("get notification channel [%d] failed: %v", req.Id, err)
		return nil, xerr.ServerErrorMsg("get notification channel failed")
	}

	return toType(row), nil
}
