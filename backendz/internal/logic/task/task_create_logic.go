package task

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskCreateLogic {
	return &TaskCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskCreateLogic) TaskCreate(req *types.CreateTaskRequest) error {
	d := req.Data
	if strings.TrimSpace(d.TypeName) == "" {
		return xerr.BadRequestMsg("typeName required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	taskType := task.TypePeriodic
	if d.Type != "" {
		taskType = task.Type(d.Type)
	}

	_, err := l.svcCtx.Ent.Task.Create().
		SetType(taskType).
		SetTypeName(d.TypeName).
		SetTaskPayload(d.TaskPayload).
		SetCronSpec(d.CronSpec).
		SetTaskOptions(schemaOptFromType(d.TaskOptions)).
		SetEnable(d.Enable).
		SetNillableRemark(nilStr(d.Remark)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now()).
		Save(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("create task failed: %v", err)
		return xerr.ServerErrorMsg("create task failed")
	}
	return nil
}

func nilStr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}