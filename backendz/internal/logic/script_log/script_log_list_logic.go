// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script_log

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/scriptlog"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptLogListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptLogListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptLogListLogic {
	return &ScriptLogListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptLogListLogic) ScriptLogList(req *types.PageRequest) (resp *types.ListScriptLogsResponse, err error) {
	q := l.svcCtx.Ent.ScriptLog.Query().Where(scriptlog.DeletedAtIsNil())

	// 搜索：按脚本名、钩子点或触发方式模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			scriptlog.Or(
				scriptlog.ScriptNameContainsFold(k),
				scriptlog.HookPointContainsFold(k),
				scriptlog.TriggerTypeContainsFold(k),
			),
		)
	}

	// 排序：最新在前
	q = q.Order(scriptlog.ByCreatedAt(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count script logs failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list script logs failed: %v", err)
		return nil, err
	}

	items := make([]types.ScriptLog, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListScriptLogsResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}
