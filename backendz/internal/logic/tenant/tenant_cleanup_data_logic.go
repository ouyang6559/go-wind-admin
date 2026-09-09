// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	userpkg "go-wind-admin/backendz/internal/ent/gen/user"
	usercredentialpkg "go-wind-admin/backendz/internal/ent/gen/usercredential"
	userrolepkg "go-wind-admin/backendz/internal/ent/gen/userrole"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantCleanupDataLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantCleanupDataLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantCleanupDataLogic {
	return &TenantCleanupDataLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TenantCleanupDataLogic) TenantCleanupData(req *types.CleanupTenantDataRequest) error {
	if req.Id <= 0 {
		return xerr.BadRequestMsg("tenant id is required")
	}

	tid := uint32(req.Id)
	existing, err := l.svcCtx.Ent.Tenant.Query().
		Where(tenantpkg.IDEQ(tid), tenantpkg.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("tenant not found")
		}
		logx.WithContext(l.ctx).Errorf("get tenant for cleanup failed: %v", err)
		return xerr.ServerErrorMsg("get tenant failed")
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	now := time.Now()
	// 保留租户记录，置为停用。
	if uerr := tx.Tenant.UpdateOneID(existing.ID).
		SetStatus(tenantpkg.StatusOff).
		Exec(l.ctx); uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("set tenant status failed: %v", uerr)
		return xerr.ServerErrorMsg("set tenant status failed")
	}

	// 软删除该租户下的用户、角色关联与凭证。
	if uerr := tx.User.Update().
		Where(userpkg.TenantIDEQ(tid), userpkg.DeletedAtIsNil()).
		SetDeletedAt(now).
		Exec(l.ctx); uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("soft delete tenant users failed: %v", uerr)
		return xerr.ServerErrorMsg("cleanup tenant users failed")
	}
	if rerr := tx.UserRole.Update().
		Where(userrolepkg.TenantIDEQ(tid), userrolepkg.DeletedAtIsNil()).
		SetDeletedAt(now).
		Exec(l.ctx); rerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("soft delete tenant user roles failed: %v", rerr)
		return xerr.ServerErrorMsg("cleanup tenant user roles failed")
	}
	if cerr := tx.UserCredential.Update().
		Where(usercredentialpkg.TenantIDEQ(tid), usercredentialpkg.DeletedAtIsNil()).
		SetDeletedAt(now).
		Exec(l.ctx); cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("soft delete tenant credentials failed: %v", cerr)
		return xerr.ServerErrorMsg("cleanup tenant credentials failed")
	}

	if cerr := tx.Commit(); cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit cleanup tx failed: %v", cerr)
		return xerr.ServerErrorMsg("commit cleanup failed")
	}

	return nil
}