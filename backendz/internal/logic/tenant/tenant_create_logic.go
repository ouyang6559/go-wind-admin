// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"
	"strings"
	"time"

	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantCreateLogic {
	return &TenantCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TenantCreateLogic) TenantCreate(req *types.CreateTenantRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" || strings.TrimSpace(d.Code) == "" {
		return xerr.BadRequestMsg("tenant name and code are required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	status := tenantpkg.StatusOn
	if d.Status != "" {
		status = tenantpkg.Status(d.Status)
	}
	typ := tenantpkg.TypePaid
	if d.Type != "" {
		typ = tenantpkg.Type(d.Type)
	}
	audit := tenantpkg.AuditStatusApproved
	if d.AuditStatus != "" {
		audit = tenantpkg.AuditStatus(d.AuditStatus)
	}

	b := l.svcCtx.Ent.Tenant.Create().
		SetNillableName(strPtr(d.Name)).
		SetNillableCode(strPtr(d.Code)).
		SetStatus(status).
		SetType(typ).
		SetAuditStatus(audit).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())

	if d.Domain != "" {
		b.SetDomain(d.Domain)
	}
	if d.LogoUrl != "" {
		b.SetLogoURL(d.LogoUrl)
	}
	if d.Industry != "" {
		b.SetIndustry(d.Industry)
	}
	if d.Remark != "" {
		b.SetRemark(d.Remark)
	}
	if d.SubscriptionPlan != "" {
		b.SetSubscriptionPlan(d.SubscriptionPlan)
	}
	if d.SubscriptionAt != "" {
		b.SetNillableSubscriptionAt(parseTime(d.SubscriptionAt))
	}
	if d.UnsubscribeAt != "" {
		b.SetNillableUnsubscribeAt(parseTime(d.UnsubscribeAt))
	}
	if d.ExpiredAt != "" {
		b.SetNillableExpiredAt(parseTime(d.ExpiredAt))
	}
	if d.AdminUserId > 0 {
		b.SetAdminUserID(uint32(d.AdminUserId))
	}

	if _, err := b.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("create tenant failed: %v", err)
		return xerr.ServerErrorMsg("create tenant failed")
	}

	return nil
}

func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

func parseTime(s string) *time.Time {
	t, err := time.Parse(time.RFC3339, s)
	if err != nil {
		return nil
	}
	return &t
}