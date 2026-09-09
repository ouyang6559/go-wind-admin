// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api

import (
	"context"

	"github.com/zeromicro/go-zero/core/logx"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
)

type ApiGetWalkRouteDataLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiGetWalkRouteDataLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiGetWalkRouteDataLogic {
	return &ApiGetWalkRouteDataLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// ApiGetWalkRouteData 获取通过 WalkRoute 获取的路由数据（用于调试）。
// backendz 网关没有 kratos 的 http.RouteWalker 遍历能力，这里兜底返回空列表，
// 保证可空数据正常返回。
func (l *ApiGetWalkRouteDataLogic) ApiGetWalkRouteData() (resp *types.ListApiResponse, err error) {
	return &types.ListApiResponse{Items: []types.Api{}}, nil
}