// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission

import (
	"context"

	"github.com/zeromicro/go-zero/core/logx"
	"go-wind-admin/backendz/internal/svc"
)

type PermissionSyncPermissionsLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionSyncPermissionsLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionSyncPermissionsLogic {
	return &PermissionSyncPermissionsLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// PermissionSyncPermissions 同步权限点。
// backendz 网关未移植 kratos 版的 MenuPermissionConverter/权限码推导与模块映射逻辑，
// 无法等价重建「由菜单+API 推导权限点」，此处提供空实现兜底，保证调用不报错。
func (l *PermissionSyncPermissionsLogic) PermissionSyncPermissions() error {
	return nil
}