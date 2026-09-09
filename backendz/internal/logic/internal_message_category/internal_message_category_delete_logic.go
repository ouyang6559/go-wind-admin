// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message_category

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessagecategory"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageCategoryDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageCategoryDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageCategoryDeleteLogic {
	return &InternalMessageCategoryDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageCategoryDeleteLogic) InternalMessageCategoryDelete(req *types.InternalMessageCategoryDeleteReq) error {
	existing, err := l.svcCtx.Ent.InternalMessageCategory.Query().
		Where(internalmessagecategory.IDEQ(uint32(req.Id)), internalmessagecategory.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("internal message category not found")
		}
		logx.WithContext(l.ctx).Errorf("get internal message category for delete failed: %v", err)
		return xerr.ServerErrorMsg("get internal message category for delete failed")
	}

	if err := l.svcCtx.Ent.InternalMessageCategory.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete internal message category failed: %v", err)
		return xerr.ServerErrorMsg("soft delete internal message category failed")
	}

	return nil
}