// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_type

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictTypeCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictTypeCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictTypeCreateLogic {
	return &DictTypeCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictTypeCreateLogic) DictTypeCreate(req *types.CreateDictTypeRequest) error {
	d := req.Data
	if strings.TrimSpace(d.TypeCode) == "" || strings.TrimSpace(d.TypeName) == "" {
		return xerr.BadRequestMsg("dict type code and name required")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	var sort uint32
	if d.SortOrder > 0 {
		sort = uint32(d.SortOrder)
	}

	builder := l.svcCtx.Ent.DictType.Create().
		SetTypeCode(d.TypeCode).
		SetTypeName(d.TypeName).
		SetIsEnabled(d.IsEnabled).
		SetNillableSortOrder(&sort).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())
	if d.TenantId > 0 {
		tid := uint32(d.TenantId)
		builder.SetNillableTenantID(&tid)
	}

	if _, err := builder.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("create dict type failed: %v", err)
		return xerr.ServerErrorMsg("create dict type failed")
	}

	return nil
}