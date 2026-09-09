// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/internalmessage"
	"go-wind-admin/backendz/internal/ent/gen/internalmessagerecipient"
	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageSendMessageLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageSendMessageLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageSendMessageLogic {
	return &InternalMessageSendMessageLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// collectTargetUserIDs 汇总定向发送目标用户 ID 集合。
func (l *InternalMessageSendMessageLogic) collectTargetUserIDs(req *types.SendMessageRequest) ([]uint32, error) {
	seen := make(map[uint32]struct{})
	ids := make([]uint32, 0, 8)
	add := func(id uint32) {
		if id == 0 {
			return
		}
		if _, ok := seen[id]; ok {
			return
		}
		seen[id] = struct{}{}
		ids = append(ids, id)
	}

	if req.RecipientUserId > 0 {
		add(uint32(req.RecipientUserId))
	}
	for _, uid := range req.TargetUserIds {
		add(uint32(uid))
	}

	if req.TargetAll {
		// 全员广播：按 ID 拉取全部未删除用户（只取 ID，避免全行进内存）。
		rows, err := l.svcCtx.Ent.User.Query().
			Where(user.DeletedAtIsNil()).
			Select(user.FieldID).
			Order(user.ByID()).
			All(l.ctx)
		if err != nil {
			return nil, err
		}
		for _, u := range rows {
			add(u.ID)
		}
	}

	return ids, nil
}

func (l *InternalMessageSendMessageLogic) InternalMessageSendMessage(req *types.SendMessageRequest) (resp *types.SendMessageResponse, err error) {
	if req.Title == "" {
		return nil, xerr.BadRequestMsg("title required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	msgType := internalmessage.TypeNotification
	if req.Type != "" {
		msgType = internalmessage.Type(req.Type)
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return nil, xerr.ServerErrorMsg("begin tx failed")
	}

	created, cerr := tx.InternalMessage.Create().
		SetTitle(req.Title).
		SetContent(req.Content).
		SetStatus(internalmessage.StatusPublished).
		SetType(msgType).
		SetSenderID(operatorID).
		SetNillableCategoryID(ptrIf(uint32(req.CategoryId))).
		SetNillableCreatedBy(&operatorID).
		Save(l.ctx)
	if cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create internal message failed: %v", cerr)
		return nil, xerr.ServerErrorMsg("create internal message failed")
	}

	// 汇总收件人
	targetIDs, uerr := l.collectTargetUserIDs(req)
	if uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("collect target users failed: %v", uerr)
		return nil, xerr.ServerErrorMsg("collect target users failed")
	}

	now := time.Now()
	for _, uid := range targetIDs {
		if _, perr := tx.InternalMessageRecipient.Create().
			SetMessageID(created.ID).
			SetRecipientUserID(uid).
			SetStatus(internalmessagerecipient.StatusReceived).
			SetReceivedAt(now).
			Save(l.ctx); perr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("create message recipient failed: msg=%d uid=%d err=%v", created.ID, uid, perr)
			return nil, xerr.ServerErrorMsg("create message recipient failed")
		}
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return nil, xerr.ServerErrorMsg("commit tx failed")
	}

	// tx7do viewer 注入：定时/异步投递在此从简——发送方在事务内同步落库，
	// SSE 实时推送是尽力而为扩展点，本网关未接入 SSE，故不做推送。

	return &types.SendMessageResponse{MessageId: int64(created.ID)}, nil
}

// ptrIf 将非零值转为指针；值为 0 返回 nil。
func ptrIf(v uint32) *uint32 {
	if v == 0 {
		return nil
	}
	return &v
}