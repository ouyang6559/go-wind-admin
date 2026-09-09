// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessage"
	"go-wind-admin/backendz/internal/ent/gen/internalmessagerecipient"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageRevokeMessageLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageRevokeMessageLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageRevokeMessageLogic {
	return &InternalMessageRevokeMessageLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageRevokeMessageLogic) InternalMessageRevokeMessage(req *types.RevokeMessageRequest) error {
	if req.MessageId <= 0 {
		return xerr.BadRequestMsg("messageId required")
	}

	// user_id > 0 为单用户撤销：仅撤销该用户的收件记录，消息本体不受影响。
	if req.UserId > 0 {
		if err := l.svcCtx.Ent.InternalMessageRecipient.Update().
			Where(
				internalmessagerecipient.MessageIDEQ(uint32(req.MessageId)),
				internalmessagerecipient.RecipientUserIDEQ(uint32(req.UserId)),
				internalmessagerecipient.DeletedAtIsNil(),
			).
			SetStatus(internalmessagerecipient.StatusRevoked).
			Exec(l.ctx); err != nil {
			logx.WithContext(l.ctx).Errorf("revoke recipient failed: msg=%d user=%d err=%v", req.MessageId, req.UserId, err)
			return xerr.ServerErrorMsg("revoke recipient failed")
		}
		return nil
	}

	// user_id == 0 为全局撤销：同一事务内撤销消息本体与全部收件记录。
	msg, err := l.svcCtx.Ent.InternalMessage.Query().
		Where(internalmessage.IDEQ(uint32(req.MessageId)), internalmessage.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("internal message not found")
		}
		logx.WithContext(l.ctx).Errorf("get internal message for revoke failed: %v", err)
		return xerr.ServerErrorMsg("get internal message for revoke failed")
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	if uerr := tx.InternalMessage.UpdateOneID(msg.ID).
		SetStatus(internalmessage.StatusRevoked).
		Exec(l.ctx); uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("revoke message failed: %v", uerr)
		return xerr.ServerErrorMsg("revoke message failed")
	}

	if rerr := tx.InternalMessageRecipient.Update().
		Where(internalmessagerecipient.MessageIDEQ(msg.ID), internalmessagerecipient.DeletedAtIsNil()).
		SetStatus(internalmessagerecipient.StatusRevoked).
		Exec(l.ctx); rerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("revoke recipients failed: %v", rerr)
		return xerr.ServerErrorMsg("revoke recipients failed")
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}