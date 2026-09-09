// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_entry

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

type DictEntryCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictEntryCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictEntryCreateLogic {
	return &DictEntryCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictEntryCreateLogic) DictEntryCreate(req *types.CreateDictEntryRequest) error {
	d := req.Data
	if d.TypeId <= 0 {
		return xerr.BadRequestMsg("dict type id required")
	}
	if strings.TrimSpace(d.EntryValue) == "" {
		return xerr.BadRequestMsg("dict entry value required")
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

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	var numericValue *int32
	if d.NumericValue != 0 {
		nv := int32(d.NumericValue)
		numericValue = &nv
	}

	builder := tx.DictEntry.Create().
		SetDictTypeID(uint32(d.TypeId)).
		SetEntryValue(d.EntryValue).
		SetIsEnabled(d.IsEnabled).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())
	if sort > 0 {
		builder.SetNillableSortOrder(&sort)
	}
	if numericValue != nil {
		builder.SetNillableNumericValue(numericValue)
	}
	if d.TenantId > 0 {
		tid := uint32(d.TenantId)
		builder.SetNillableTenantID(&tid)
	}

	created, cerr := builder.Save(l.ctx)
	if cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create dict entry failed: %v", cerr)
		return xerr.ServerErrorMsg("create dict entry failed")
	}

	if len(d.I18n) > 0 {
		if ierr := replaceI18n(l.ctx, tx, created.ID, operatorID, d.I18n); ierr != nil {
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