// Package pkassist 应用启动辅助：种子初始化（默认管理员 + 完整默认数据）。
package pkassist

import (
	"context"
	"fmt"

	"entgo.io/ent/privacy"
	gocrudviewer "github.com/tx7do/go-crud/viewer"

	"go-wind-admin/backendz/internal/ent/gen/membership"
	"go-wind-admin/backendz/internal/ent/gen/membershiprole"
	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/ent/gen/permission"
	"go-wind-admin/backendz/internal/ent/gen/permissiongroup"
	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/ent/gen/rolepermission"
	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/ent/gen/userrole"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/pkg/viewer"
	"go-wind-admin/backendz/internal/svc"
)

// Seed 启动播种：每类默认数据各自做 count==0 守卫，幂等可重复调用。
// - 默认管理员（admin / admin）不存在时创建；
// - 语言 / 权限组 / 权限点 / 角色+元数据+权限 / 菜单 各表为空时才播种；
// - 给 admin 绑定 platform:admin 角色（user_role + membership + membership_role）。
func Seed(s *svc.ServiceContext) error {
	// 平台上下文：使 tenant 隐私策略放行（IsPlatformContext），否则缺 viewer 会 fail-closed。
	ctx := gocrudviewer.WithContext(context.Background(), viewer.Default)
	// 种子写入注入 Allow 决策上下文，避免被 schema 策略拦截。
	allowCtx := privacy.DecisionContext(ctx, privacy.Allow)

	if err := seedAdminUser(ctx, allowCtx, s); err != nil {
		return err
	}

	if err := seedLanguages(ctx, allowCtx, s); err != nil {
		return err
	}

	groupIDByName, err := seedPermissionGroups(ctx, allowCtx, s)
	if err != nil {
		return err
	}

	permIDByCode, err := seedPermissions(ctx, allowCtx, s, groupIDByName)
	if err != nil {
		return err
	}

	roleIDByCode, err := seedRoles(ctx, allowCtx, s, permIDByCode)
	if err != nil {
		return err
	}

	if err := seedMenus(ctx, allowCtx, s); err != nil {
		return err
	}

	if err := seedUserBindings(ctx, allowCtx, s, roleIDByCode); err != nil {
		return err
	}

	return nil
}

// seedAdminUser 创建默认管理员及其凭证（幂等）。
func seedAdminUser(ctx, allowCtx context.Context, s *svc.ServiceContext) error {
	exists, err := s.Ent.User.Query().Where(user.UsernameEQ("admin")).Exist(ctx)
	if err != nil {
		return err
	}
	if exists {
		return nil
	}

	hash, err := password.Hash("admin")
	if err != nil {
		return err
	}

	u, err := s.Ent.User.Create().
		SetTenantID(0).
		SetUsername("admin").
		SetNickname("Administrator").
		SetStatus(user.StatusNormal).
		Save(allowCtx)
	if err != nil {
		return fmt.Errorf("seed admin user: %w", err)
	}

	_, err = s.Ent.UserCredential.Create().
		SetTenantID(0).
		SetUserID(u.ID).
		SetIdentityType(usercredential.IdentityTypeUsername).
		SetIdentifier("admin").
		SetCredentialType(usercredential.CredentialTypePasswordHash).
		SetCredential(hash).
		SetIsPrimary(true).
		SetStatus(usercredential.StatusEnabled).
		Save(allowCtx)
	if err != nil {
		return fmt.Errorf("seed admin credential: %w", err)
	}
	return nil
}

// seedLanguages 播种 7 种默认语言，language_code 全局唯一。
func seedLanguages(ctx, allowCtx context.Context, s *svc.ServiceContext) error {
	n, err := s.Ent.Language.Query().Count(ctx)
	if err != nil {
		return err
	}
	if n > 0 {
		return nil
	}

	for _, l := range defaultLanguages {
		if _, err := s.Ent.Language.Create().
			SetLanguageCode(l.code).
			SetLanguageName(l.name).
			SetNativeName(l.native).
			SetIsDefault(l.isDefault).
			SetSortOrder(l.sortOrder).
			SetIsEnabled(true).
			Save(allowCtx); err != nil {
			return fmt.Errorf("seed language %s: %w", l.code, err)
		}
	}
	return nil
}

// seedPermissionGroups 播种 5 条树形权限组，返回 name->ID 映射（供权限点绑定）。
func seedPermissionGroups(ctx, allowCtx context.Context, s *svc.ServiceContext) (map[string]uint32, error) {
	idByName := map[string]uint32{}

	n, err := s.Ent.PermissionGroup.Query().Count(ctx)
	if err != nil {
		return nil, err
	}
	if n > 0 {
		grps, err := s.Ent.PermissionGroup.Query().All(ctx)
		if err != nil {
			return nil, err
		}
		for _, g := range grps {
			if g.Name != nil {
				idByName[*g.Name] = g.ID
			}
		}
		return idByName, nil
	}

	for _, pg := range defaultPermissionGroups {
		c := s.Ent.PermissionGroup.Create().
			SetName(pg.name).
			SetModule(pg.module).
			SetPath(pg.path).
			SetDescription(pg.name).
			SetSortOrder(pg.sortOrder).
			SetStatus(permissiongroup.StatusOn)
		if pg.parentName != "" {
			pid, ok := idByName[pg.parentName]
			if !ok {
				return nil, fmt.Errorf("seed permission group %s: parent %s missing", pg.name, pg.parentName)
			}
			c.SetParentID(pid)
		}
		saved, err := c.Save(allowCtx)
		if err != nil {
			return nil, fmt.Errorf("seed permission group %s: %w", pg.name, err)
		}
		idByName[pg.name] = saved.ID
	}
	return idByName, nil
}

// seedPermissions 播种 5 条权限点，返回 code->ID 映射（供角色权限绑定）。
// 权限点 group_id 关联到真实权限组自动生成 ID。
func seedPermissions(ctx, allowCtx context.Context, s *svc.ServiceContext, groupIDByName map[string]uint32) (map[string]uint32, error) {
	codeToID := map[string]uint32{}

	n, err := s.Ent.Permission.Query().Count(ctx)
	if err != nil {
		return nil, err
	}
	if n > 0 {
		perms, err := s.Ent.Permission.Query().All(ctx)
		if err != nil {
			return nil, err
		}
		for _, p := range perms {
			if p.Code != nil {
				codeToID[*p.Code] = p.ID
			}
		}
		return codeToID, nil
	}

	for _, p := range defaultPermissions {
		gid, ok := groupIDByName[p.groupName]
		if !ok {
			return nil, fmt.Errorf("seed permission %s: group %s missing", p.code, p.groupName)
		}
		saved, err := s.Ent.Permission.Create().
			SetName(p.name).
			SetCode(p.code).
			SetGroupID(gid).
			SetDescription(p.desc).
			SetStatus(permission.StatusOn).
			Save(allowCtx)
		if err != nil {
			return nil, fmt.Errorf("seed permission %s: %w", p.code, err)
		}
		codeToID[p.code] = saved.ID
	}
	return codeToID, nil
}

// seedRoles 播种 2 条系统角色 + 角色元数据 + 角色-权限绑定，返回 code->ID 映射。
// 权限绑定使用权限点播种后的真实自增 ID（不硬编码 1/2/4）。
func seedRoles(ctx, allowCtx context.Context, s *svc.ServiceContext, permIDByCode map[string]uint32) (map[string]uint32, error) {
	idByCode := map[string]uint32{}

	n, err := s.Ent.Role.Query().Count(ctx)
	if err != nil {
		return nil, err
	}
	if n > 0 {
		roles, err := s.Ent.Role.Query().All(ctx)
		if err != nil {
			return nil, err
		}
		for _, r := range roles {
			if r.Code != nil {
				idByCode[*r.Code] = r.ID
			}
		}
		return idByCode, nil
	}

	for _, sr := range defaultRoles {
		saved, err := s.Ent.Role.Create().
			SetTenantID(0).
			SetName(sr.name).
			SetCode(sr.code).
			SetDescription(sr.desc).
			SetIsProtected(sr.isProtected).
			SetType(sr.typ).
			SetSortOrder(sr.sortOrder).
			SetStatus(role.StatusOn).
			Save(allowCtx)
		if err != nil {
			return nil, fmt.Errorf("seed role %s: %w", sr.code, err)
		}
		idByCode[sr.code] = saved.ID

		// 角色元数据
		if md, ok := metadataByRole(sr.code); ok {
			mc := s.Ent.RoleMetadata.Create().
				SetTenantID(0).
				SetRoleID(saved.ID).
				SetIsTemplate(md.isTemplate).
				SetSyncPolicy(md.syncPolicy).
				SetScope(md.scope)
			if md.templateFor != "" {
				mc.SetTemplateFor(md.templateFor)
			}
			if md.isTemplate {
				mc.SetTemplateVersion(1)
			}
			if _, err := mc.Save(allowCtx); err != nil {
				return nil, fmt.Errorf("seed role metadata %s: %w", sr.code, err)
			}
		}

		// 角色-权限绑定
		for _, code := range sr.permissionCodes {
			pid, ok := permIDByCode[code]
			if !ok {
				return nil, fmt.Errorf("seed role %s: permission %s missing", sr.code, code)
			}
			if _, err := s.Ent.RolePermission.Create().
				SetTenantID(0).
				SetRoleID(saved.ID).
				SetPermissionID(pid).
				SetStatus(rolepermission.StatusOn).
				Save(allowCtx); err != nil {
				return nil, fmt.Errorf("seed role_permission %s->%s: %w", sr.code, code, err)
			}
		}
	}
	return idByCode, nil
}

// seedMenus 播种 33 条树形菜单（type/parent/path/redirect/component/module/meta/status）。
func seedMenus(ctx, allowCtx context.Context, s *svc.ServiceContext) error {
	n, err := s.Ent.Menu.Query().Count(ctx)
	if err != nil {
		return err
	}
	if n > 0 {
		return nil
	}

	idByName := map[string]uint32{}
	for _, sm := range defaultMenus {
		c := s.Ent.Menu.Create().
			SetName(sm.name).
			SetType(sm.typ).
			SetPath(sm.path).
			SetComponent(sm.component).
			SetStatus(menu.StatusOn)
		if sm.parentName != "" {
			pid, ok := idByName[sm.parentName]
			if !ok {
				return fmt.Errorf("seed menu %s: parent %s missing", sm.name, sm.parentName)
			}
			c.SetParentID(pid)
		}
		if sm.redirect != "" {
			c.SetRedirect(sm.redirect)
		}
		if sm.hasModule {
			c.SetModule(sm.module)
		}
		if sm.meta != nil {
			c.SetMeta(sm.meta)
		}
		saved, err := c.Save(allowCtx)
		if err != nil {
			return fmt.Errorf("seed menu %s: %w", sm.name, err)
		}
		idByName[sm.name] = saved.ID
	}
	return nil
}

// seedUserBindings 给 admin 用户绑定 platform:admin 角色（user_role + membership + membership_role）。
func seedUserBindings(ctx, allowCtx context.Context, s *svc.ServiceContext, roleIDByCode map[string]uint32) error {
	admin, err := s.Ent.User.Query().Where(user.UsernameEQ("admin")).Only(ctx)
	if err != nil {
		return fmt.Errorf("find admin user for binding: %w", err)
	}

	platformAdminID, ok := roleIDByCode[roleCodePlatformAdmin]
	if !ok {
		return fmt.Errorf("seed user binding: role %s missing", roleCodePlatformAdmin)
	}

	// user_role 绑定
	urExists, err := s.Ent.UserRole.Query().Where(
		userrole.TenantID(0),
		userrole.UserIDEQ(admin.ID),
		userrole.RoleIDEQ(platformAdminID),
	).Exist(ctx)
	if err != nil {
		return err
	}
	if !urExists {
		if _, err := s.Ent.UserRole.Create().
			SetTenantID(0).
			SetUserID(admin.ID).
			SetRoleID(platformAdminID).
			SetIsPrimary(true).
			SetStatus(userrole.StatusActive).
			Save(allowCtx); err != nil {
			return fmt.Errorf("seed user_role admin->%s: %w", roleCodePlatformAdmin, err)
		}
	}

	// membership + membership_role 绑定
	mExists, err := s.Ent.Membership.Query().Where(
		membership.TenantID(0),
		membership.UserIDEQ(admin.ID),
	).Exist(ctx)
	if err != nil {
		return err
	}
	if !mExists {
		m, err := s.Ent.Membership.Create().
			SetTenantID(0).
			SetUserID(admin.ID).
			SetRoleID(platformAdminID).
			SetIsPrimary(true).
			SetStatus(membership.StatusActive).
			Save(allowCtx)
		if err != nil {
			return fmt.Errorf("seed membership admin: %w", err)
		}
		if _, err := s.Ent.MembershipRole.Create().
			SetTenantID(0).
			SetMembershipID(m.ID).
			SetRoleID(platformAdminID).
			SetIsPrimary(true).
			SetStatus(membershiprole.StatusActive).
			Save(allowCtx); err != nil {
			return fmt.Errorf("seed membership_role admin: %w", err)
		}
	}

	return nil
}
