// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package admin_portal

import (
	"context"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type AdminPortalGetMyPermissionCodeLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAdminPortalGetMyPermissionCodeLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AdminPortalGetMyPermissionCodeLogic {
	return &AdminPortalGetMyPermissionCodeLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *AdminPortalGetMyPermissionCodeLogic) AdminPortalGetMyPermissionCode() (resp *types.ListPermissionCodeResponse, err error) {
	claim, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return nil, xerr.UnauthorizedMsg("未登录")
	}

	// 查询用户关联的角色
	roleIDs, err := roleIDsOfUser(l.ctx, l.svcCtx.Ent, claim.UserID)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("query user roles failed: %v", err)
		return nil, xerr.ServerErrorMsg("query user roles failed")
	}

	// 查询角色关联的权限
	permissionIDs, err := permissionIDsOfRoles(l.ctx, l.svcCtx.Ent, roleIDs)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("query role permissions failed: %v", err)
		return nil, xerr.ServerErrorMsg("query role permissions failed")
	}

	// 查询权限编码
	codes, err := permissionCodesOf(l.ctx, l.svcCtx.Ent, permissionIDs)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("query permission codes failed: %v", err)
		return nil, xerr.ServerErrorMsg("query permission codes failed")
	}

	return &types.ListPermissionCodeResponse{Codes: codes}, nil
}
