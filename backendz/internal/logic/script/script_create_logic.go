// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptCreateLogic {
	return &ScriptCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptCreateLogic) ScriptCreate(req *types.CreateScriptRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" {
		return xerr.BadRequestMsg("script name required")
	}

	// 名称唯一校验
	exists, err := l.svcCtx.Ent.Script.Query().
		Where(script.NameEQ(strings.TrimSpace(d.Name)), script.DeletedAtIsNil()).
		Exist(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("check script name exists failed: %v", err)
		return xerr.ServerErrorMsg("check script name failed")
	}
	if exists {
		return xerr.BadRequestMsg("script name already exists")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	now := time.Now()
	_, cerr := l.svcCtx.Ent.Script.Create().
		SetNillableName(strPtr(strings.TrimSpace(d.Name))).
		SetNillableLanguage(langPtr(d.Language)).
		SetNillableHookPoint(strPtr(d.HookPoint)).
		SetNillableSource(strPtr(d.Source)).
		SetNillablePriority(prioPtr(d.Priority)).
		SetNillableDescription(strPtr(d.Description)).
		SetNillableCritical(boolPtr(d.Critical)).
		SetNillableVersion(uintPtr(d.Version)).
		SetNillableIsEnabled(boolPtr(d.IsEnabled)).
		SetNillableCreatedBy(&operatorID).
		SetNillableUpdatedBy(&operatorID).
		SetCreatedAt(now).
		SetUpdatedAt(now).
		Save(l.ctx)
	if cerr != nil {
		logx.WithContext(l.ctx).Errorf("create script failed: %v", cerr)
		return xerr.ServerErrorMsg("create script failed")
	}

	return nil
}

func prioPtr(v int64) *int32 {
	p := int32(v)
	return &p
}

func uintPtr(v int64) *uint32 {
	p := uint32(v)
	return &p
}

func boolPtr(v bool) *bool {
	return &v
}
