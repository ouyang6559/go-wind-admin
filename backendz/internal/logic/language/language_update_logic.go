// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package language

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/language"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LanguageUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLanguageUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LanguageUpdateLogic {
	return &LanguageUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LanguageUpdateLogic) LanguageUpdate(req *types.UpdateLanguageRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Language.Query().
		Where(language.IDEQ(uint32(req.Id)), language.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("language not found")
		}
		logx.WithContext(l.ctx).Errorf("get language for update failed: %v", err)
		return xerr.ServerErrorMsg("get language for update failed")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.Language.UpdateOneID(existing.ID).
		SetUpdatedBy(operatorID).
		SetUpdatedAt(time.Now())
	// language_code 为 Immutable 字段，创建后不可更改
	if d.LanguageName != "" {
		upd.SetLanguageName(d.LanguageName)
	}
	if d.NativeName != "" {
		upd.SetNativeName(d.NativeName)
	}
	if d.SortOrder > 0 {
		upd.SetSortOrder(uint32(d.SortOrder))
	}
	upd.SetIsDefault(d.IsDefault).
		SetIsEnabled(d.IsEnabled)

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update language failed: %v", uerr)
		return xerr.ServerErrorMsg("update language failed")
	}

	return nil
}