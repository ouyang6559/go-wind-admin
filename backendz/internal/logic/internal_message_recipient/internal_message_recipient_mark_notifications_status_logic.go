// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package internal_message_recipient

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/internalmessagerecipient"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageRecipientMarkNotificationsStatusLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageRecipientMarkNotificationsStatusLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageRecipientMarkNotificationsStatusLogic {
	return &InternalMessageRecipientMarkNotificationsStatusLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// statusFromCode 将新状态码（0 SENT / 1 RECEIVED / 2 READ / 3 REVOKED / 4 DELETED）
// 映射为 ent 状态枚举。非法码返回 ok=false。
func statusFromCode(code int64) (internalmessagerecipient.Status, bool) {
	switch code {
	case 0:
		return internalmessagerecipient.StatusSent, true
	case 1:
		return internalmessagerecipient.StatusReceived, true
	case 2:
		return internalmessagerecipient.StatusRead, true
	case 3:
		return internalmessagerecipient.StatusRevoked, true
	case 4:
		return internalmessagerecipient.StatusDeleted, true
	default:
		return "", false
	}
}

// InternalMessageRecipientMarkNotificationsStatus 标记指定用户的某些收件消息的状态。
// 状态迁移到 READ 时回填 read_at，迁移到 RECEIVED(已接收) 时回填 received_at；
// 已处于目标状态的记录不重写（StatusNEQ 守卫）。
func (l *InternalMessageRecipientMarkNotificationsStatusLogic) InternalMessageRecipientMarkNotificationsStatus(req *types.MarkNotificationsStatusRequest) error {
	if len(req.RecipientIds) == 0 {
		return xerr.BadRequestMsg("invalid parameter")
	}
	if req.UserId == 0 {
		return xerr.BadRequestMsg("invalid parameter")
	}

	newStatus, ok := statusFromCode(req.NewStatus)
	if !ok {
		return xerr.BadRequestMsg("invalid status")
	}

	ids := make([]uint32, 0, len(req.RecipientIds))
	for _, id := range req.RecipientIds {
		if id > 0 {
			ids = append(ids, uint32(id))
		}
	}
	if len(ids) == 0 {
		return xerr.BadRequestMsg("invalid parameter")
	}

	now := time.Now()
	var readAt, receivedAt *time.Time
	switch req.NewStatus {
	case 2: // READ
		readAt = &now
	case 1: // RECEIVED
		receivedAt = &now
	}

	if err := l.svcCtx.Ent.InternalMessageRecipient.Update().
		Where(
			internalmessagerecipient.IDIn(ids...),
			internalmessagerecipient.RecipientUserIDEQ(uint32(req.UserId)),
			internalmessagerecipient.StatusNEQ(newStatus),
		).
		SetStatus(newStatus).
		SetNillableReadAt(readAt).
		SetNillableReceivedAt(receivedAt).
		SetUpdatedAt(now).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("mark notifications status to [%s] for user [%d] failed: %v", newStatus, req.UserId, err)
		return xerr.ServerErrorMsg("mark notifications status failed")
	}

	return nil
}