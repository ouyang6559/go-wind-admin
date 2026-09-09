// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api

import (
	"context"

	"github.com/zeromicro/go-zero/core/logx"
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
// backendz 网关不内置 OpenAPI/WalkRoute 资源来源，无法像 kratos 版那样自动重建 API 列表，
// 此处提供空实现兜底，保证调用不报错；如需真实同步请自行注入数据源。
func (l *ApiSyncApisLogic) ApiSyncApis() error {
	return nil
}