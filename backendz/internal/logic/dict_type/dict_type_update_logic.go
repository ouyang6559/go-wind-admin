// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_type

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/dicttype"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictTypeUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictTypeUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictTypeUpdateLogic {
	return &DictTypeUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictTypeUpdateLogic) DictTypeUpdate(req *types.UpdateDictTypeRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.DictType.Query().
		Where(dicttype.IDEQ(uint32(req.Id)), dicttype.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("dict type not found")
		}
		logx.WithContext(l.ctx).Errorf("get dict type for update failed: %v", err)
		return xerr.ServerErrorMsg("get dict type for update failed")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.DictType.UpdateOneID(existing.ID).
		SetUpdatedBy(operatorID).
		SetUpdatedAt(time.Now())
	// type_code 为 Immutable 字段，创建后不可更改
	if d.TypeName != "" {
		upd.SetTypeName(d.TypeName)
	}
	if d.SortOrder > 0 {
		upd.SetSortOrder(uint32(d.SortOrder))
	}
	upd.SetIsEnabled(d.IsEnabled)

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update dict type failed: %v", uerr)
		return xerr.ServerErrorMsg("update dict type failed")
	}

	return nil
}