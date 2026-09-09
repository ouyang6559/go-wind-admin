// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package org_unit

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/orgunit"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type OrgUnitUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOrgUnitUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OrgUnitUpdateLogic {
	return &OrgUnitUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *OrgUnitUpdateLogic) OrgUnitUpdate(req *types.UpdateOrgUnitRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.OrgUnit.Query().
		Where(orgunit.IDEQ(uint32(req.Id)), orgunit.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("org unit not found")
		}
		logx.WithContext(l.ctx).Errorf("get org unit for update failed: %v", err)
		return xerr.ServerErrorMsg("get org unit for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.OrgUnit.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if d.Code != "" {
		upd.SetCode(d.Code)
	}
	if d.Remark != "" {
		upd.SetRemark(d.Remark)
	}
	if d.Description != "" {
		upd.SetDescription(d.Description)
	}
	if d.Status != "" {
		upd.SetStatus(orgunit.Status(d.Status))
	}
	if d.Type != "" {
		upd.SetType(orgunit.Type(d.Type))
	}
	if d.SortOrder > 0 {
		upd.SetSortOrder(uint32(d.SortOrder))
	}
	if len(d.BusinessScopes) > 0 {
		upd.SetBusinessScopes(d.BusinessScopes)
	}
	if len(d.PermissionTags) > 0 {
		upd.SetPermissionTags(d.PermissionTags)
	}
	if d.LeaderId > 0 {
		upd.SetLeaderID(uint32(d.LeaderId))
	}
	if d.ContactUserId > 0 {
		upd.SetContactUserID(uint32(d.ContactUserId))
	}
	if st, ok := parseTime(d.StartAt); ok {
		upd.SetStartAt(st)
	}
	if et, ok := parseTime(d.EndAt); ok {
		upd.SetEndAt(et)
	}

	if _, err := upd.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("update org unit failed: %v", err)
		return xerr.ServerErrorMsg("update org unit failed")
	}

	return nil
}
