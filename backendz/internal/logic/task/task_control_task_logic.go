package task

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen/task"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TaskControlTaskLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTaskControlTaskLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TaskControlTaskLogic {
	return &TaskControlTaskLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TaskControlTaskLogic) TaskControlTask(req *types.ControlTaskRequest) error {
	if _, ok := middleware.ClaimsFromContext(l.ctx); !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}
	tn := strings.TrimSpace(req.TypeName)
	if tn == "" {
		return xerr.BadRequestMsg("typeName required")
	}

	existing, err := l.svcCtx.Ent.Task.Query().
		Where(task.TypeNameEQ(tn), task.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		return xerr.NotFoundMsg("task not found")
	}

	enable := true
	switch strings.ToLower(req.ControlType) {
	case "stop", "disable", "pause":
		enable = false
	case "", "start", "enable", "resume":
		enable = true
	default:
		return xerr.BadRequestMsg("invalid controlType")
	}

	if _, uerr := l.svcCtx.Ent.Task.UpdateOneID(existing.ID).
		SetEnable(enable).
		Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("control task failed: %v", uerr)
		return xerr.ServerErrorMsg("control task failed")
	}
	return nil
}