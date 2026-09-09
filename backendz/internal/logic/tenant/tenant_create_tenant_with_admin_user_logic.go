// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"
	"time"

	rolepkg "go-wind-admin/backendz/internal/ent/gen/role"
	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	userpkg "go-wind-admin/backendz/internal/ent/gen/user"
	usercredentialpkg "go-wind-admin/backendz/internal/ent/gen/usercredential"
	userrolepkg "go-wind-admin/backendz/internal/ent/gen/userrole"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantCreateTenantWithAdminUserLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantCreateTenantWithAdminUserLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantCreateTenantWithAdminUserLogic {
	return &TenantCreateTenantWithAdminUserLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// TenantCreateTenantWithAdminUser 创建租户及其管理员用户（事务）。
// 流程：查重 → 建租户 → 建租户管理员角色 → 建用户 → 建用户名凭证 → 建用户角色关联 → 回填 tenant.admin_user_id。
func (l *TenantCreateTenantWithAdminUserLogic) TenantCreateTenantWithAdminUser(req *types.CreateTenantWithAdminUserRequest) error {
	tReq := req.Tenant
	uReq := req.User
	if tReq.Name == "" || tReq.Code == "" {
		return xerr.BadRequestMsg("tenant name and code are required")
	}
	if uReq.Username == "" {
		return xerr.BadRequestMsg("username is required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	// 租户 code/name 查重。
	if exist, err := tenantExistsDB(l.ctx, l.svcCtx, tReq.Code, tReq.Name); err != nil {
		logx.WithContext(l.ctx).Errorf("check tenant exists err: %v", err)
		return err
	} else if exist {
		return xerr.BadRequestMsg("tenant with given code or name already exists")
	}

	// 密码哈希（为空用默认口令）。
	plain := req.Password
	if plain == "" {
		plain = "123456"
	}
	hash, err := password.Hash(plain)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("hash password failed: %v", err)
		return xerr.ServerErrorMsg("hash password failed")
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	// 1. 创建租户
	tenantRow, terr := tx.Tenant.Create().
		SetNillableName(strPtr(tReq.Name)).
		SetNillableCode(strPtr(tReq.Code)).
		SetStatus(tenantStatusDefault(tReq.Status)).
		SetType(tenantTypeDefault(tReq.Type)).
		SetNillableCreatedBy(&operatorID).
		Save(l.ctx)
	if terr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create tenant err: %v", terr)
		return xerr.ServerErrorMsg("create tenant failed")
	}

	// 2. 创建租户管理员角色（租户级管理角色模板）
	roleRow, rerr := tx.Role.Create().
		SetName("租户管理员").
		SetCode("TENANT_ADMIN").
		SetType(rolepkg.TypeTenant).
		SetStatus(rolepkg.StatusOn).
		SetNillableCreatedBy(&operatorID).
		Save(l.ctx)
	if rerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create tenant admin role err: %v", rerr)
		return xerr.ServerErrorMsg("create tenant admin role failed")
	}

	// 3. 创建管理员用户
	userRow, uerr := tx.User.Create().
		SetNillableTenantID(&tenantRow.ID).
		SetUsername(uReq.Username).
		SetStatus(userStatusDefault(uReq.Status)).
		SetNillableCreatedBy(&operatorID).
		Save(l.ctx)
	if uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create tenant admin user err: %v", uerr)
		return xerr.ServerErrorMsg("create tenant admin user failed")
	}

	// 4. 创建用户名凭证
	if _, cerr := tx.UserCredential.Create().
		SetNillableTenantID(&tenantRow.ID).
		SetUserID(userRow.ID).
		SetIdentityType(usercredentialpkg.IdentityTypeUsername).
		SetIdentifier(uReq.Username).
		SetCredentialType(usercredentialpkg.CredentialTypePasswordHash).
		SetCredential(hash).
		SetIsPrimary(true).
		SetStatus(usercredentialpkg.StatusEnabled).
		Save(l.ctx); cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create tenant admin credential err: %v", cerr)
		return xerr.ServerErrorMsg("create user credential failed")
	}

	// 5. 绑定管理员角色
	now := time.Now()
	if _, aerr := tx.UserRole.Create().
		SetNillableTenantID(&tenantRow.ID).
		SetUserID(userRow.ID).
		SetRoleID(roleRow.ID).
		SetIsPrimary(true).
		SetStatus(userrolepkg.StatusActive).
		SetNillableAssignedBy(&operatorID).
		SetNillableAssignedAt(&now).
		Save(l.ctx); aerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("bind tenant admin role err: %v", aerr)
		return xerr.ServerErrorMsg("bind user role failed")
	}

	// 6. 回填租户管理员
	if _, aerr := tx.Tenant.UpdateOneID(tenantRow.ID).
		SetAdminUserID(userRow.ID).
		Save(l.ctx); aerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("assign tenant admin err: %v", aerr)
		return xerr.ServerErrorMsg("assign tenant admin failed")
	}

	if cerr := tx.Commit(); cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", cerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}

// tenantExistsDB 检查 code 或 name 的租户是否已存在。
func tenantExistsDB(ctx context.Context, svcCtx *svc.ServiceContext, code, name string) (bool, error) {
	if code == "" && name == "" {
		return false, nil
	}
	var q = svcCtx.Ent.Tenant.Query().Where(tenantpkg.DeletedAtIsNil())
	if code != "" && name != "" {
		q = q.Where(tenantpkg.Or(tenantpkg.CodeEQ(code), tenantpkg.NameEQ(name)))
	} else if code != "" {
		q = q.Where(tenantpkg.CodeEQ(code))
	} else {
		q = q.Where(tenantpkg.NameEQ(name))
	}
	count, err := q.Count(ctx)
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

func tenantStatusDefault(s string) tenantpkg.Status {
	if s == "" {
		return tenantpkg.StatusOn
	}
	return tenantpkg.Status(s)
}

func tenantTypeDefault(s string) tenantpkg.Type {
	if s == "" {
		return tenantpkg.TypePaid
	}
	return tenantpkg.Type(s)
}

func userStatusDefault(s string) userpkg.Status {
	if s == "" {
		return userpkg.StatusNormal
	}
	return userpkg.Status(s)
}