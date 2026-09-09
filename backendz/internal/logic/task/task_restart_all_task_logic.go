package task

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskRestartAllTaskLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskRestartAllTaskLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskRestartAllTaskLogic {
	return &TaskRestartAllTaskLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskRestartAllTaskLogic) TaskRestartAllTask() (resp *types.RestartAllTaskResponse, err error) {
	if _, ok := middleware.ClaimsFromContext(l.ctx); !ok {
		return nil, xerr.UnauthorizedMsg("unauthorized")
	}
	n, err := l.svcCtx.Ent.Task.Update().
		Where(task.DeletedAtIsNil()).
		SetEnable(true).
		Save(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("restart all tasks failed: %v", err)
		return nil, xerr.ServerErrorMsg("restart all tasks failed")
	}
	return &types.RestartAllTaskResponse{Count: int64(n)}, nil
}