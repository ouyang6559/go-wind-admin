package task

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskDeleteLogic {
	return &TaskDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskDeleteLogic) TaskDelete(req *types.TaskDeleteReq) error {
	if req.Id <= 0 {
		return xerr.BadRequestMsg("id required")
	}
	existing, err := l.svcCtx.Ent.Task.Query().
		Where(task.IDEQ(uint32(req.Id)), task.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("task not found")
		}
		logx.WithContext(l.ctx).Errorf("get task for delete failed: %v", err)
		return xerr.ServerErrorMsg("get task for delete failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	if err := l.svcCtx.Ent.Task.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		SetDeletedBy(operatorID).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete task failed: %v", err)
		return xerr.ServerErrorMsg("soft delete task failed")
	}
	return nil
}