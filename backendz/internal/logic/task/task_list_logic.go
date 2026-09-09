package task

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskListLogic {
	return &TaskListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskListLogic) TaskList(req *types.PageRequest) (resp *types.ListTaskResponse, err error) {
	q := l.svcCtx.Ent.Task.Query().Where(task.DeletedAtIsNil())

	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(task.Or(
			task.TypeNameContainsFold(k),
			task.CronSpecContainsFold(k),
		))
	}
	if t := strings.TrimSpace(req.Filter); t != "" {
		q = q.Where(task.TypeEQ(task.Type(t)))
	}

	q = q.Order(task.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count tasks failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list tasks failed: %v", err)
		return nil, err
	}

	items := make([]types.Task, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}
	return &types.ListTaskResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}