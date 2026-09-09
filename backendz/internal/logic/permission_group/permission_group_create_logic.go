// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission_group

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/permissiongroup"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionGroupCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionGroupCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionGroupCreateLogic {
	return &PermissionGroupCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionGroupCreateLogic) PermissionGroupCreate(req *types.CreatePermissionGroupRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" {
		return xerr.BadRequestMsg("permission group name required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	status := permissiongroup.StatusOn
	if d.Status != "" {
		status = permissiongroup.Status(d.Status)
	}

	var sort uint32
	if d.SortOrder > 0 {
		sort = uint32(d.SortOrder)
	}

	create := l.svcCtx.Ent.PermissionGroup.Create().
		SetName(d.Name).
		SetStatus(status).
		SetSortOrder(sort).
		SetNillableDescription(nilStr(d.Description)).
		SetNillableModule(nilStr(d.Module)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())
	if d.ParentId > 0 {
		create = create.SetParentID(uint32(d.ParentId))
	}
	if d.Path != "" {
		create = create.SetNillablePath(nilStr(d.Path))
	}

	if _, cerr := create.Save(l.ctx); cerr != nil {
		logx.WithContext(l.ctx).Errorf("create permission group failed: %v", cerr)
		return xerr.ServerErrorMsg("create permission group failed")
	}

	return nil
}

func nilStr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}