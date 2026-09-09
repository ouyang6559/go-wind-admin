package task

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskListTaskTypeNameLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskListTaskTypeNameLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskListTaskTypeNameLogic {
	return &TaskListTaskTypeNameLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskListTaskTypeNameLogic) TaskListTaskTypeName() (resp *types.ListTaskTypeNameResponse, err error) {
	rows, err := l.svcCtx.Ent.Task.Query().
		Where(task.DeletedAtIsNil()).
		Select(task.FieldTypeName).
		All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list task type names failed: %v", err)
		return nil, err
	}
	seen := make(map[string]struct{})
	names := make([]string, 0, len(rows))
	for _, r := range rows {
		n := strings.TrimSpace(stringsVal(r.TypeName))
		if n == "" {
			continue
		}
		if _, dup := seen[n]; dup {
			continue
		}
		seen[n] = struct{}{}
		names = append(names, n)
	}
	return &types.ListTaskTypeNameResponse{TypeNames: names}, nil
}

func stringsVal(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}