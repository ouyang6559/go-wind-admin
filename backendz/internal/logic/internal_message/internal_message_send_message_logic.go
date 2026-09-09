// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message

import (
	"context"
	"encoding/json"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
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
	recipients := make([]*gen.InternalMessageRecipient, 0, len(targetIDs))
	for _, uid := range targetIDs {
		rec, perr := tx.InternalMessageRecipient.Create().
			SetMessageID(created.ID).
			SetRecipientUserID(uid).
			SetStatus(internalmessagerecipient.StatusReceived).
			SetReceivedAt(now).
			Save(l.ctx)
		if perr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("create message recipient failed: msg=%d uid=%d err=%v", created.ID, uid, perr)
			return nil, xerr.ServerErrorMsg("create message recipient failed")
		}
		recipients = append(recipients, rec)
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return nil, xerr.ServerErrorMsg("commit tx failed")
	}

	// SSE 实时通知（尽力而为）：事务提交成功后再按收件人推送 notification 事件，
	// 与 backend/SSE 端点一致（用户不在线/缓冲满时跳过，不阻塞发送方）。
	title := ""
	if created.Title != nil {
		title = *created.Title
	}
	content := ""
	if created.Content != nil {
		content = *created.Content
	}
	for _, rec := range recipients {
		recipientID := uint32(0)
		if rec.RecipientUserID != nil {
			recipientID = *rec.RecipientUserID
		}
		status := ""
		if rec.Status != nil {
			status = string(*rec.Status)
		}
		payload, merr := json.Marshal(notificationPayload{
			ID:              int64(rec.ID),
			MessageID:       int64(created.ID),
			RecipientUserID: int64(recipientID),
			Status:          status,
			ReceivedAt:      now.Format(time.RFC3339),
			Title:           title,
			Content:         content,
			SenderID:        int64(operatorID),
			Type:            string(msgType),
		})
		if merr != nil {
			logx.WithContext(l.ctx).Errorf("marshal sse notification payload failed: %v", merr)
			continue
		}
		if n := l.svcCtx.Sse.Publish(recipientID, payload); n == 0 {
			logx.WithContext(l.ctx).Debugf("sse publish skipped (no online subscriber): user=%d msg=%d", recipientID, created.ID)
		}
	}

	return &types.SendMessageResponse{MessageId: int64(created.ID)}, nil
}

// notificationPayload 是 SSE notification 事件的 data 结构，
// 字段语义与 backend 的 InternalMessageRecipient 保持一致。
type notificationPayload struct {
	ID              int64  `json:"id"`
	MessageID       int64  `json:"messageId"`
	RecipientUserID int64  `json:"recipientUserId"`
	Status          string `json:"status"`
	ReceivedAt      string `json:"receivedAt"`
	Title           string `json:"title"`
	Content         string `json:"content"`
	SenderID        int64  `json:"createdBy"`
	Type            string `json:"type"`
}

// ptrIf 将非零值转为指针；值为 0 返回 nil。
func ptrIf(v uint32) *uint32 {
	if v == 0 {
		return nil
	}
	return &v
}
