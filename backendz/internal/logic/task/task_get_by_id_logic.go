package task

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskGetByIdLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskGetByIdLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskGetByIdLogic {
	return &TaskGetByIdLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskGetByIdLogic) TaskGetById(req *types.TaskGetByIdReq) (resp *types.Task, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}
	e, err := l.svcCtx.Ent.Task.Query().
		Where(task.IDEQ(uint32(req.Id)), task.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("task not found")
		}
		logx.WithContext(l.ctx).Errorf("get task by id failed: %v", err)
		return nil, err
	}
	return toType(e), nil
}