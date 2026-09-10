// Package pkassist 应用启动辅助：种子初始化（默认管理员 + 完整默认数据）。
package pkassist

import (
	"context"
	"fmt"

	"entgo.io/ent/privacy"
	gocrudviewer "github.com/tx7do/go-crud/viewer"

	"go-wind-admin/backendz/internal/ent/gen/language"
	"go-wind-admin/backendz/internal/ent/gen/membership"
	"go-wind-admin/backendz/internal/ent/gen/membershiprole"
	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/ent/gen/permission"
	"go-wind-admin/backendz/internal/ent/gen/permissiongroup"
	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/ent/gen/rolemetadata"
	"go-wind-admin/backendz/internal/ent/gen/rolepermission"
	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/ent/gen/userrole"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/pkg/viewer"
	"go-wind-admin/backendz/internal/svc"
)

// Seed 启动播种：逐条按唯一键（username/code/name 等）幂等补齐默认数据。
// 注意：不使用「表 count==0 才播种」的粗粒度守卫——库中可能存在部分手工数据
// （如测试角色）导致 count>0 而漏播默认数据。每项默认数据独立判断存在性，
// 已存在则跳过（并复用其 ID），缺失才创建，可重复调用。
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

	// API 资源表同步（增量补齐，幂等）：从已注册路由清单写入 sys_apis。
	// 与 kratos 空表自动同步行为一致，使权限组/接口管理页可见真实接口清单。
	if err := syncApis(ctx, allowCtx, s); err != nil {
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

// seedLanguages 逐条播种默认语言（按 language_code 判重）。
func seedLanguages(ctx, allowCtx context.Context, s *svc.ServiceContext) error {
	for _, l := range defaultLanguages {
		exists, err := s.Ent.Language.Query().Where(language.LanguageCodeEQ(l.code)).Exist(ctx)
		if err != nil {
			return err
		}
		if exists {
			continue
		}
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

// seedPermissionGroups 逐条播种树形权限组（按 name 判重），返回 name->ID 映射。
func seedPermissionGroups(ctx, allowCtx context.Context, s *svc.ServiceContext) (map[string]uint32, error) {
	idByName := map[string]uint32{}

	// 先加载全部现有分组，复用已存在 ID（含手工创建的同名组）。
	grps, err := s.Ent.PermissionGroup.Query().All(ctx)
	if err != nil {
		return nil, err
	}
	for _, g := range grps {
		if g.Name != nil {
			idByName[*g.Name] = g.ID
		}
	}

	for _, pg := range defaultPermissionGroups {
		if _, ok := idByName[pg.name]; ok {
			continue
		}
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

// seedPermissions 逐条播种权限点（按 code 判重），返回 code->ID 映射（供角色权限绑定）。
func seedPermissions(ctx, allowCtx context.Context, s *svc.ServiceContext, groupIDByName map[string]uint32) (map[string]uint32, error) {
	codeToID := map[string]uint32{}

	perms, err := s.Ent.Permission.Query().All(ctx)
	if err != nil {
		return nil, err
	}
	for _, p := range perms {
		if p.Code != nil {
			codeToID[*p.Code] = p.ID
		}
	}

	for _, p := range defaultPermissions {
		if _, ok := codeToID[p.code]; ok {
			continue
		}
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

// seedRoles 逐条播种系统角色（按 code 判重）+ 角色元数据 + 角色-权限绑定。
// 权限绑定使用权限点播种后的真实自增 ID（不硬编码 1/2/4）。
func seedRoles(ctx, allowCtx context.Context, s *svc.ServiceContext, permIDByCode map[string]uint32) (map[string]uint32, error) {
	idByCode := map[string]uint32{}

	roles, err := s.Ent.Role.Query().All(ctx)
	if err != nil {
		return nil, err
	}
	for _, r := range roles {
		if r.Code != nil {
			idByCode[*r.Code] = r.ID
		}
	}

	for _, sr := range defaultRoles {
		id, ok := idByCode[sr.code]
		if !ok {
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
			id = saved.ID
			idByCode[sr.code] = id
		}

		// 角色元数据：按 role_id 判重
		if md, ok := metadataByRole(sr.code); ok {
			mdExists, err := s.Ent.RoleMetadata.Query().Where(rolemetadata.RoleIDEQ(id)).Exist(ctx)
			if err != nil {
				return nil, err
			}
			if !mdExists {
				mc := s.Ent.RoleMetadata.Create().
					SetTenantID(0).
					SetRoleID(id).
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
		}

		// 角色-权限绑定：按 (role_id, permission_id) 判重
		for _, code := range sr.permissionCodes {
			pid, ok := permIDByCode[code]
			if !ok {
				return nil, fmt.Errorf("seed role %s: permission %s missing", sr.code, code)
			}
			rpExists, err := s.Ent.RolePermission.Query().Where(
				rolepermission.RoleIDEQ(id),
				rolepermission.PermissionIDEQ(pid),
			).Exist(ctx)
			if err != nil {
				return nil, err
			}
			if rpExists {
				continue
			}
			if _, err := s.Ent.RolePermission.Create().
				SetTenantID(0).
				SetRoleID(id).
				SetPermissionID(pid).
				SetStatus(rolepermission.StatusOn).
				Save(allowCtx); err != nil {
				return nil, fmt.Errorf("seed role_permission %s->%s: %w", sr.code, code, err)
			}
		}
	}
	return idByCode, nil
}

// seedMenus 逐条播种树形菜单（按 name 判重）。
func seedMenus(ctx, allowCtx context.Context, s *svc.ServiceContext) error {
	idByName := map[string]uint32{}

	menus, err := s.Ent.Menu.Query().All(ctx)
	if err != nil {
		return err
	}
	for _, m := range menus {
		if m.Name != nil {
			idByName[*m.Name] = m.ID
		}
	}

	for _, sm := range defaultMenus {
		if _, ok := idByName[sm.name]; ok {
			continue
		}
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
