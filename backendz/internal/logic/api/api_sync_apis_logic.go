// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api

import (
	"context"

	"github.com/zeromicro/go-zero/core/logx"
	"go-wind-admin/backendz/internal/pkassist"
	"go-wind-admin/backendz/internal/svc"
)

type ApiSyncApisLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiSyncApisLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiSyncApisLogic {
	return &ApiSyncApisLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// ApiSyncApis 同步 API 资源。
// 与启动播种共用 pkassist.SyncApis：从已注册路由清单（routes.go/colon_routes.go）
// 增量补齐 sys_apis（按 method+path 判重），使管理页「接口同步」真实生效。
func (l *ApiSyncApisLogic) ApiSyncApis() error {
	return pkassist.SyncApis(l.svcCtx)
}
