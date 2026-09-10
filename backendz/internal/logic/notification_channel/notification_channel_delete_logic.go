// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/notificationchannel"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

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
	if req.Id <= 0 {
		return xerr.BadRequestMsg("notification channel id required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	// 软删除：仅置 deleted_at/deleted_by
	n, err := l.svcCtx.Ent.NotificationChannel.Update().
		Where(notificationchannel.IDEQ(uint32(req.Id)), notificationchannel.DeletedAtIsNil()).
		SetNillableDeletedBy(&operatorID).
		SetDeletedAt(time.Now()).
		Save(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("notification channel not found")
		}
		logx.WithContext(l.ctx).Errorf("delete notification channel [%d] failed: %v", req.Id, err)
		return xerr.ServerErrorMsg("delete notification channel failed")
	}
	if n == 0 {
		return xerr.NotFoundMsg("notification channel not found")
	}

	return nil
}
