// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission_group

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/permissiongroup"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionGroupUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionGroupUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionGroupUpdateLogic {
	return &PermissionGroupUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionGroupUpdateLogic) PermissionGroupUpdate(req *types.UpdatePermissionGroupRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.PermissionGroup.Query().
		Where(permissiongroup.IDEQ(uint32(req.Id)), permissiongroup.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("permission group not found")
		}
		logx.WithContext(l.ctx).Errorf("get permission group for update failed: %v", err)
		return xerr.ServerErrorMsg("get permission group for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.PermissionGroup.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if d.Status != "" {
		upd.SetStatus(permissiongroup.Status(d.Status))
	}
	if d.Module != "" {
		upd.SetModule(d.Module)
	}
	if d.Description != "" {
		upd.SetDescription(d.Description)
	}
	if d.SortOrder > 0 {
		upd.SetSortOrder(uint32(d.SortOrder))
	}
	if d.ParentId > 0 {
		upd.SetParentID(uint32(d.ParentId))
	}
	if d.Path != "" {
		upd.SetPath(d.Path)
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update permission group failed: %v", uerr)
		return xerr.ServerErrorMsg("update permission group failed")
	}

	return nil
}