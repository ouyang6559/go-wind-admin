package task

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskGetByTypeNameLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskGetByTypeNameLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskGetByTypeNameLogic {
	return &TaskGetByTypeNameLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskGetByTypeNameLogic) TaskGetByTypeName(req *types.TaskGetByTypeNameReq) (resp *types.Task, err error) {
	tn := strings.TrimSpace(req.TypeName)
	if tn == "" {
		return nil, xerr.BadRequestMsg("typeName required")
	}
	q := l.svcCtx.Ent.Task.Query().
		Where(task.TypeNameEQ(tn), task.DeletedAtIsNil())
	if req.Id > 0 {
		q = q.Where(task.IDEQ(uint32(req.Id)))
	}
	e, err := q.Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("task not found")
		}
		logx.WithContext(l.ctx).Errorf("get task by typeName failed: %v", err)
		return nil, err
	}
	return toType(e), nil
}