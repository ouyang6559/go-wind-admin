// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package role

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type RoleGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewRoleGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *RoleGetLogic {
	return &RoleGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *RoleGetLogic) RoleGet(req *types.RoleGetReq) (resp *types.Role, err error) {
	var q = l.svcCtx.Ent.Role.Query().Where(role.DeletedAtIsNil())
	if req.Code != "" {
		q = q.Where(role.CodeEQ(req.Code))
	} else if req.Id > 0 {
		q = q.Where(role.IDEQ(uint32(req.Id)))
	} else {
		return nil, xerr.BadRequestMsg("id or code required")
	}

	e, err := q.Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("role not found")
		}
		logx.WithContext(l.ctx).Errorf("get role failed: %v", err)
		return nil, err
	}

	return toType(l.ctx, l.svcCtx.Ent, e)
}