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

type TaskUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskUpdateLogic {
	return &TaskUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskUpdateLogic) TaskUpdate(req *types.UpdateTaskRequest) error {
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
		logx.WithContext(l.ctx).Errorf("get task for update failed: %v", err)
		return xerr.ServerErrorMsg("get task for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	d := req.Data
	upd := l.svcCtx.Ent.Task.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Type != "" {
		upd.SetType(task.Type(d.Type))
	}
	if d.TypeName != "" {
		upd.SetTypeName(d.TypeName)
	}
	if d.TaskPayload != "" {
		upd.SetTaskPayload(d.TaskPayload)
	}
	if d.CronSpec != "" {
		upd.SetCronSpec(d.CronSpec)
	}
	upd.SetTaskOptions(schemaOptFromType(d.TaskOptions))
	upd.SetEnable(d.Enable)
	if d.Remark != "" {
		upd.SetRemark(d.Remark)
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update task failed: %v", uerr)
		return xerr.ServerErrorMsg("update task failed")
	}
	return nil
}