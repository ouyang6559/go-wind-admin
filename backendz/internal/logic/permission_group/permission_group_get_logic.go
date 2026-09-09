// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission_group

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/permissiongroup"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionGroupGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionGroupGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionGroupGetLogic {
	return &PermissionGroupGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionGroupGetLogic) PermissionGroupGet(req *types.PermissionGroupGetReq) (resp *types.PermissionGroup, err error) {
	e, err := l.svcCtx.Ent.PermissionGroup.Query().
		Where(permissiongroup.IDEQ(uint32(req.Id)), permissiongroup.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("permission group not found")
		}
		logx.WithContext(l.ctx).Errorf("get permission group failed: %v", err)
		return nil, err
	}

	return toType(e), nil
}