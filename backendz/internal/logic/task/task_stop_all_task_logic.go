package task

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskStopAllTaskLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskStopAllTaskLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskStopAllTaskLogic {
	return &TaskStopAllTaskLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskStopAllTaskLogic) TaskStopAllTask() error {
	if _, ok := middleware.ClaimsFromContext(l.ctx); !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}
	if _, err := l.svcCtx.Ent.Task.Update().
		Where(task.DeletedAtIsNil()).
		SetEnable(false).
		Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("stop all tasks failed: %v", err)
		return xerr.ServerErrorMsg("stop all tasks failed")
	}
	return nil
}