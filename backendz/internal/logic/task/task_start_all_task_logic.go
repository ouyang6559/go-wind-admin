package task

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskStartAllTaskLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskStartAllTaskLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskStartAllTaskLogic {
	return &TaskStartAllTaskLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskStartAllTaskLogic) TaskStartAllTask() error {
	if _, ok := middleware.ClaimsFromContext(l.ctx); !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}
	if _, err := l.svcCtx.Ent.Task.Update().
		Where(task.DeletedAtIsNil()).
		SetEnable(true).
		Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("start all tasks failed: %v", err)
		return xerr.ServerErrorMsg("start all tasks failed")
	}
	return nil
}