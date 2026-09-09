// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message_category

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessagecategory"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageCategoryUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageCategoryUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageCategoryUpdateLogic {
	return &InternalMessageCategoryUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageCategoryUpdateLogic) InternalMessageCategoryUpdate(req *types.UpdateInternalMessageCategoryRequest) error {
	existing, err := l.svcCtx.Ent.InternalMessageCategory.Query().
		Where(internalmessagecategory.IDEQ(uint32(req.Id)), internalmessagecategory.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("internal message category not found")
		}
		logx.WithContext(l.ctx).Errorf("get internal message category for update failed: %v", err)
		return xerr.ServerErrorMsg("get internal message category for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	d := req.Data
	upd := l.svcCtx.Ent.InternalMessageCategory.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if d.Code != "" {
		upd.SetCode(d.Code)
	}
	if d.IconUrl != "" {
		upd.SetIconURL(d.IconUrl)
	}
	if d.SortOrder > 0 {
		upd.SetSortOrder(uint32(d.SortOrder))
	}
	upd.SetIsEnabled(d.IsEnabled)

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update internal message category failed: %v", uerr)
		return xerr.ServerErrorMsg("update internal message category failed")
	}

	return nil
}