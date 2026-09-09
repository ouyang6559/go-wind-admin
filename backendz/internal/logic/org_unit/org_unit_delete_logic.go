// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package org_unit

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/orgunit"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type OrgUnitDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOrgUnitDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OrgUnitDeleteLogic {
	return &OrgUnitDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *OrgUnitDeleteLogic) OrgUnitDelete(req *types.OrgUnitDeleteReq) error {
	existing, err := l.svcCtx.Ent.OrgUnit.Query().
		Where(orgunit.IDEQ(uint32(req.Id)), orgunit.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("org unit not found")
		}
		logx.WithContext(l.ctx).Errorf("get org unit for delete failed: %v", err)
		return xerr.ServerErrorMsg("get org unit for delete failed")
	}

	// 存在子节点时禁止删除，保护层级树。
	if ch, err := l.svcCtx.Ent.OrgUnit.Query().
		Where(orgunit.ParentID(existing.ID), orgunit.DeletedAtIsNil()).
		Count(l.ctx); err == nil && ch > 0 {
		return xerr.ForbiddenMsg("org unit has children, cannot delete")
	}

	if err := l.svcCtx.Ent.OrgUnit.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete org unit failed: %v", err)
		return xerr.ServerErrorMsg("soft delete org unit failed")
	}

	return nil
}
