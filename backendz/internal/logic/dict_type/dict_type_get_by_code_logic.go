// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_type

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/dicttype"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictTypeGetByCodeLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictTypeGetByCodeLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictTypeGetByCodeLogic {
	return &DictTypeGetByCodeLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictTypeGetByCodeLogic) DictTypeGetByCode(req *types.DictTypeGetByCodeReq) (resp *types.DictType, err error) {
	if req.Code == "" {
		return nil, xerr.BadRequestMsg("code required")
	}

	e, err := l.svcCtx.Ent.DictType.Query().
		Where(dicttype.TypeCodeEQ(req.Code), dicttype.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("dict type not found")
		}
		logx.WithContext(l.ctx).Errorf("get dict type by code failed: %v", err)
		return nil, err
	}

	return toType(e), nil
}