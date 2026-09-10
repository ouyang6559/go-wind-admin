// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptUpdateLogic {
	return &ScriptUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptUpdateLogic) ScriptUpdate(req *types.UpdateScriptRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Script.Query().
		Where(script.IDEQ(uint32(req.Id)), script.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("script not found")
		}
		logx.WithContext(l.ctx).Errorf("get script for update failed: %v", err)
		return xerr.ServerErrorMsg("get script for update failed")
	}

	// 改名时校验唯一性
	if n := strings.TrimSpace(d.Name); n != "" && !strings.EqualFold(n, stdStr(existing.Name)) {
		exists, eerr := l.svcCtx.Ent.Script.Query().
			Where(script.NameEQ(n), script.DeletedAtIsNil(), script.IDNEQ(existing.ID)).
			Exist(l.ctx)
		if eerr != nil {
			logx.WithContext(l.ctx).Errorf("check script name exists during update failed: %v", eerr)
			return xerr.ServerErrorMsg("check script name failed")
		}
		if exists {
			return xerr.BadRequestMsg("script name already exists")
		}
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	// 版本自增（热更新指纹）
	nextVersion := uint32(1)
	if existing.Version != nil {
		nextVersion = *existing.Version + 1
	}

	upd := l.svcCtx.Ent.Script.UpdateOneID(existing.ID).
		SetNillableUpdatedBy(&operatorID).
		SetUpdatedAt(time.Now()).
		SetNillableVersion(&nextVersion)

	if strings.TrimSpace(d.Name) != "" {
		upd.SetNillableName(strPtr(strings.TrimSpace(d.Name)))
	}
	if d.Language != "" {
		upd.SetNillableLanguage(langPtr(d.Language))
	}
	if d.HookPoint != "" {
		upd.SetNillableHookPoint(strPtr(d.HookPoint))
	}
	if d.Source != "" {
		upd.SetNillableSource(strPtr(d.Source))
	}
	if d.Description != "" {
		upd.SetNillableDescription(strPtr(d.Description))
	}
	if d.Priority != 0 {
		upd.SetNillablePriority(prioPtr(d.Priority))
	}
	if d.Critical {
		upd.SetNillableCritical(boolPtr(true))
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update script failed: %v", uerr)
		return xerr.ServerErrorMsg("update script failed")
	}

	return nil
}

func stdStr(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}
