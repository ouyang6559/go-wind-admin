// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message_recipient

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/internalmessagerecipient"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageRecipientMarkNotificationAsReadLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageRecipientMarkNotificationAsReadLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageRecipientMarkNotificationAsReadLogic {
	return &InternalMessageRecipientMarkNotificationAsReadLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageRecipientMarkNotificationAsReadLogic) InternalMessageRecipientMarkNotificationAsRead(req *types.MarkNotificationAsReadRequest) error {
	var userID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		userID = c.UserID
	}
	if userID == 0 {
		return xerr.ForbiddenMsg("user context required")
	}

	now := time.Now()
	builder := l.svcCtx.Ent.InternalMessageRecipient.Update().
		Where(
			internalmessagerecipient.RecipientUserIDEQ(userID),
			internalmessagerecipient.StatusNEQ(internalmessagerecipient.StatusRead),
			internalmessagerecipient.DeletedAtIsNil(),
		)
	if len(req.RecipientIds) > 0 {
		ids := make([]uint32, 0, len(req.RecipientIds))
		for _, id := range req.RecipientIds {
			if id > 0 {
				ids = append(ids, uint32(id))
			}
		}
		if len(ids) > 0 {
			builder = builder.Where(internalmessagerecipient.IDIn(ids...))
		}
	}

	if err := builder.
		SetStatus(internalmessagerecipient.StatusRead).
		SetReadAt(now).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("mark notification as read failed: %v", err)
		return xerr.ServerErrorMsg("mark notification as read failed")
	}

	return nil
}