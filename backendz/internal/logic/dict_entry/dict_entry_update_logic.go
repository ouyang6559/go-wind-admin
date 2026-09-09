// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_entry

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/dictentry"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictEntryUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictEntryUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictEntryUpdateLogic {
	return &DictEntryUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictEntryUpdateLogic) DictEntryUpdate(req *types.UpdateDictEntryRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.DictEntry.Query().
		Where(dictentry.IDEQ(uint32(req.Id)), dictentry.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("dict entry not found")
		}
		logx.WithContext(l.ctx).Errorf("get dict entry for update failed: %v", err)
		return xerr.ServerErrorMsg("get dict entry for update failed")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	upd := tx.DictEntry.UpdateOneID(existing.ID).
		SetUpdatedBy(operatorID).
		SetUpdatedAt(time.Now())
	if d.EntryValue != "" {
		upd.SetEntryValue(d.EntryValue)
	}
	if d.NumericValue != 0 {
		nv := int32(d.NumericValue)
		upd.SetNillableNumericValue(&nv)
	}
	if d.SortOrder > 0 {
		upd.SetSortOrder(uint32(d.SortOrder))
	}
	if d.TypeId > 0 {
		upd.SetDictTypeID(uint32(d.TypeId))
	}
	upd.SetIsEnabled(d.IsEnabled)

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("update dict entry failed: %v", uerr)
		return xerr.ServerErrorMsg("update dict entry failed")
	}

	// 提供 i18n 时整体重建多语言数据
	if len(d.I18n) > 0 {
		if ierr := replaceI18n(l.ctx, tx, existing.ID, operatorID, d.I18n); ierr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("replace dict entry i18n failed: %v", ierr)
			return xerr.ServerErrorMsg("replace dict entry i18n failed")
		}
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}