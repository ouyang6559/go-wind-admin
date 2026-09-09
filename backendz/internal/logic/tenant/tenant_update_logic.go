// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantUpdateLogic {
	return &TenantUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TenantUpdateLogic) TenantUpdate(req *types.UpdateTenantRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Tenant.Query().
		Where(tenantpkg.IDEQ(uint32(req.Id)), tenantpkg.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("tenant not found")
		}
		logx.WithContext(l.ctx).Errorf("get tenant for update failed: %v", err)
		return xerr.ServerErrorMsg("get tenant for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.Tenant.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if d.Code != "" {
		upd.SetCode(d.Code)
	}
	if d.Domain != "" {
		upd.SetDomain(d.Domain)
	}
	if d.LogoUrl != "" {
		upd.SetLogoURL(d.LogoUrl)
	}
	if d.Industry != "" {
		upd.SetIndustry(d.Industry)
	}
	if d.Remark != "" {
		upd.SetRemark(d.Remark)
	}
	if d.SubscriptionPlan != "" {
		upd.SetSubscriptionPlan(d.SubscriptionPlan)
	}
	if d.Status != "" {
		upd.SetStatus(tenantpkg.Status(d.Status))
	}
	if d.Type != "" {
		upd.SetType(tenantpkg.Type(d.Type))
	}
	if d.AuditStatus != "" {
		upd.SetAuditStatus(tenantpkg.AuditStatus(d.AuditStatus))
	}
	if d.SubscriptionAt != "" {
		if t := parseTime(d.SubscriptionAt); t != nil {
			upd.SetSubscriptionAt(*t)
		}
	}
	if d.UnsubscribeAt != "" {
		if t := parseTime(d.UnsubscribeAt); t != nil {
			upd.SetUnsubscribeAt(*t)
		}
	}
	if d.ExpiredAt != "" {
		if t := parseTime(d.ExpiredAt); t != nil {
			upd.SetExpiredAt(*t)
		}
	}
	if d.AdminUserId > 0 {
		upd.SetAdminUserID(uint32(d.AdminUserId))
	}

	if _, err := upd.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("update tenant failed: %v", err)
		return xerr.ServerErrorMsg("update tenant failed")
	}

	return nil
}