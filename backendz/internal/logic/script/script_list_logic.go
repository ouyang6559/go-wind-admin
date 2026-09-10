// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptListLogic {
	return &ScriptListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptListLogic) ScriptList(req *types.PageRequest) (resp *types.ListScriptsResponse, err error) {
	q := l.svcCtx.Ent.Script.Query().Where(script.DeletedAtIsNil())

	// 搜索：按 name 或 hook_point 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			script.Or(
				script.NameContainsFold(k),
				script.HookPointContainsFold(k),
			),
		)
	}

	// 排序：优先级升序 + 创建时间倒序
	q = q.Order(script.ByPriority(sql.OrderAsc()), script.ByCreatedAt(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count scripts failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list scripts failed: %v", err)
		return nil, err
	}

	items := make([]types.Script, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListScriptsResponse{
		Items: items,
		Total: int64(total),
	}, nil
}
