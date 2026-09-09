// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package org_unit

import (
	"context"
	"strconv"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/orgunit"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type OrgUnitCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOrgUnitCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OrgUnitCreateLogic {
	return &OrgUnitCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *OrgUnitCreateLogic) OrgUnitCreate(req *types.CreateOrgUnitRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" {
		return xerr.BadRequestMsg("org unit name required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	// path 计算：根节点 "/"，非根节点 "/1/2/"。
	parentPath := "/"
	var parentID uint32
	if d.ParentId > 0 {
		parentID = uint32(d.ParentId)
		p, err := l.svcCtx.Ent.OrgUnit.Query().
			Where(orgunit.IDEQ(parentID), orgunit.DeletedAtIsNil()).
			Select(orgunit.FieldPath).
			Only(l.ctx)
		if err != nil {
			if gen.IsNotFound(err) {
				return xerr.BadRequestMsg("parent org unit not found")
			}
			logx.WithContext(l.ctx).Errorf("get parent org unit failed: %v", err)
			return xerr.ServerErrorMsg("get parent org unit failed")
		}
		parentPath = std.Str(p.Path)
	}
	path := parentPath + strconv.FormatUint(uint64(parentID), 10) + "/"

	orgStatus := orgunit.StatusOn
	if d.Status != "" {
		orgStatus = orgunit.Status(d.Status)
	}
	orgType := orgunit.TypeDepartment
	if d.Type != "" {
		orgType = orgunit.Type(d.Type)
	}

	var sort uint32
	if d.SortOrder > 0 {
		sort = uint32(d.SortOrder)
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	b := tx.OrgUnit.Create().
		SetName(d.Name).
		SetPath(path).
		SetNillableCode(strPtr(d.Code)).
		SetNillableSortOrder(&sort).
		SetType(orgType).
		SetStatus(orgStatus).
		SetBusinessScopes(d.BusinessScopes).
		SetPermissionTags(d.PermissionTags).
		SetNillableDescription(strPtr(d.Description)).
		SetNillableRemark(strPtr(d.Remark)).
		SetNillableLeaderID(uiPtr(d.LeaderId)).
		SetNillableContactUserID(uiPtr(d.ContactUserId)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())

	// 可选字段：缓冲为 NULL 时自动落库默认/空值。
	if d.ParentId > 0 {
		b.SetNillableParentID(uiPtr(d.ParentId))
	}
	if d.IsLegalEntity {
		b.SetNillableIsLegalEntity(&d.IsLegalEntity)
	}
	if d.ExternalId != "" {
		b.SetNillableExternalID(strPtr(d.ExternalId))
	}
	if d.RegistrationNumber != "" {
		b.SetNillableRegistrationNumber(strPtr(d.RegistrationNumber))
	}
	if d.TaxId != "" {
		b.SetNillableTaxID(strPtr(d.TaxId))
	}
	if d.LegalEntityOrgId > 0 {
		b.SetNillableLegalEntityOrgID(uiPtr(d.LegalEntityOrgId))
	}
	if d.Address != "" {
		b.SetNillableAddress(strPtr(d.Address))
	}
	if d.Phone != "" {
		b.SetNillablePhone(strPtr(d.Phone))
	}
	if d.Email != "" {
		b.SetNillableEmail(strPtr(d.Email))
	}
	if d.Timezone != "" {
		b.SetNillableTimezone(strPtr(d.Timezone))
	}
	if d.Country != "" {
		b.SetNillableCountry(strPtr(d.Country))
	}
	if d.Latitude != 0 {
		b.SetNillableLatitude(f64Ptr(d.Latitude))
	}
	if d.Longitude != 0 {
		b.SetNillableLongitude(f64Ptr(d.Longitude))
	}
	if st, ok := parseTime(d.StartAt); ok {
		b.SetNillableStartAt(&st)
	}
	if et, ok := parseTime(d.EndAt); ok {
		b.SetNillableEndAt(&et)
	}

	if _, cerr := b.Save(l.ctx); cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create org unit failed: %v", cerr)
		return xerr.ServerErrorMsg("create org unit failed")
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}

// parseTime 解析 RFC3339 时间字符串。
func parseTime(s string) (time.Time, bool) {
	if s == "" {
		return time.Time{}, false
	}
	t, err := time.Parse(time.RFC3339, s)
	if err != nil {
		return time.Time{}, false
	}
	return t, true
}
