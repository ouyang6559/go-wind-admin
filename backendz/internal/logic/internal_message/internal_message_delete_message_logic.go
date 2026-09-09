// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessage"
	"go-wind-admin/backendz/internal/ent/gen/internalmessagerecipient"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageDeleteMessageLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageDeleteMessageLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageDeleteMessageLogic {
	return &InternalMessageDeleteMessageLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageDeleteMessageLogic) InternalMessageDeleteMessage(req *types.InternalMessageDeleteMessageReq) error {
	existing, err := l.svcCtx.Ent.InternalMessage.Query().
		Where(internalmessage.IDEQ(uint32(req.Id)), internalmessage.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("internal message not found")
		}
		logx.WithContext(l.ctx).Errorf("get internal message for delete failed: %v", err)
		return xerr.ServerErrorMsg("get internal message for delete failed")
	}

	// 消息本体与全部收件记录同事务软删，避免留下孤儿收件行。
	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	now := time.Now()
	if derr := tx.InternalMessage.UpdateOneID(existing.ID).
		SetDeletedAt(now).
		Exec(l.ctx); derr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("soft delete internal message failed: %v", derr)
		return xerr.ServerErrorMsg("soft delete internal message failed")
	}

	if rerr := tx.InternalMessageRecipient.Update().
		Where(internalmessagerecipient.MessageIDEQ(existing.ID), internalmessagerecipient.DeletedAtIsNil()).
		SetDeletedAt(now).
		Exec(l.ctx); rerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("soft delete recipients failed: %v", rerr)
		return xerr.ServerErrorMsg("soft delete recipients failed")
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}