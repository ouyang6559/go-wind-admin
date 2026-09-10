// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptListHookPointsLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptListHookPointsLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptListHookPointsLogic {
	return &ScriptListHookPointsLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptListHookPointsLogic) ScriptListHookPoints() (resp *types.ListHookPointsResponse, err error) {
	rows, err := l.svcCtx.Ent.Script.Query().
		Where(script.DeletedAtIsNil(), script.HookPointNotNil()).
		All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list scripts for hook points failed: %v", err)
		return nil, err
	}

	// 按钩子点聚合挂载数
	counts := make(map[string]int)
	langSet := make(map[string]struct{})
	var names []string
	for _, r := range rows {
		if r.HookPoint == nil || *r.HookPoint == "" {
			continue
		}
		hp := *r.HookPoint
		if _, ok := counts[hp]; !ok {
			names = append(names, hp)
		}
		counts[hp]++

		if r.Language != nil {
			langSet[string(*r.Language)] = struct{}{}
		}
	}

	items := make([]types.HookPoint, 0, len(names))
	for _, name := range names {
		items = append(items, types.HookPoint{
			Name:        name,
			Description: "platform hook point",
			ScriptCount: int64(counts[name]),
		})
	}

	languages := make([]string, 0, len(langSet))
	for lg := range langSet {
		languages = append(languages, lg)
	}

	return &types.ListHookPointsResponse{
		Items:     items,
		Languages: languages,
	}, nil
}
