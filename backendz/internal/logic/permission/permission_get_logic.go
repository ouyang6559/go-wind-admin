// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/permission"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionGetLogic {
	return &PermissionGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionGetLogic) PermissionGet(req *types.PermissionGetReq) (resp *types.Permission, err error) {
	var q = l.svcCtx.Ent.Permission.Query().Where(permission.DeletedAtIsNil())
	if req.Code != "" {
		q = q.Where(permission.CodeEQ(req.Code))
	} else if req.Id > 0 {
		q = q.Where(permission.IDEQ(uint32(req.Id)))
	} else {
		return nil, xerr.BadRequestMsg("id or code required")
	}

	e, err := q.Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("permission not found")
		}
		logx.WithContext(l.ctx).Errorf("get permission failed: %v", err)
		return nil, err
	}

	return toType(l.ctx, l.svcCtx.Ent, e)
}