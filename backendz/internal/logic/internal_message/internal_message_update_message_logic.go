// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessage"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageUpdateMessageLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageUpdateMessageLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageUpdateMessageLogic {
	return &InternalMessageUpdateMessageLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageUpdateMessageLogic) InternalMessageUpdateMessage(req *types.UpdateInternalMessageRequest) error {
	existing, err := l.svcCtx.Ent.InternalMessage.Query().
		Where(internalmessage.IDEQ(uint32(req.Id)), internalmessage.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("internal message not found")
		}
		logx.WithContext(l.ctx).Errorf("get internal message for update failed: %v", err)
		return xerr.ServerErrorMsg("get internal message for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.InternalMessage.UpdateOneID(existing.ID).SetUpdatedBy(operatorID)
	d := req.Data
	if d.Title != "" {
		upd.SetTitle(d.Title)
	}
	if d.Content != "" {
		upd.SetContent(d.Content)
	}
	if d.Status != "" {
		upd.SetStatus(internalmessage.Status(d.Status))
	}
	if d.Type != "" {
		upd.SetType(internalmessage.Type(d.Type))
	}
	if d.CategoryId > 0 {
		upd.SetCategoryID(uint32(d.CategoryId))
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update internal message failed: %v", uerr)
		return xerr.ServerErrorMsg("update internal message failed")
	}

	return nil
}