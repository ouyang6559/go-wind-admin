// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ScriptGetByIdLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewScriptGetByIdLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ScriptGetByIdLogic {
	return &ScriptGetByIdLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ScriptGetByIdLogic) ScriptGetById(req *types.GetScriptReq) (resp *types.Script, err error) {
	row, err := l.svcCtx.Ent.Script.Query().
		Where(script.IDEQ(uint32(req.Id)), script.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("script not found")
		}
		logx.WithContext(l.ctx).Errorf("get script by id failed: %v", err)
		return nil, err
	}

	return toType(row), nil
}
