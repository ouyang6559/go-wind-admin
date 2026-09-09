// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package org_unit

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/orgunit"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type OrgUnitGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOrgUnitGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OrgUnitGetLogic {
	return &OrgUnitGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *OrgUnitGetLogic) OrgUnitGet(req *types.OrgUnitGetReq) (resp *types.OrgUnit, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.OrgUnit.Query().
		Where(orgunit.IDEQ(uint32(req.Id)), orgunit.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("org unit not found")
		}
		logx.WithContext(l.ctx).Errorf("get org unit failed: %v", err)
		return nil, err
	}

	return toType(l.ctx, l.svcCtx.Ent, e), nil
}
