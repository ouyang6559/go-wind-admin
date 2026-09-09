// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package language

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LanguageBatchCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLanguageBatchCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LanguageBatchCreateLogic {
	return &LanguageBatchCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LanguageBatchCreateLogic) LanguageBatchCreate(req *types.BatchCreateLanguagesRequest) error {
	if len(req.Items) == 0 {
		return xerr.BadRequestMsg("languages items required")
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

	for i := range req.Items {
		d := req.Items[i]
		if d.LanguageCode == "" || d.LanguageName == "" {
			_ = tx.Rollback()
			return xerr.BadRequestMsg("language code and name required")
		}

		var sort uint32
		if d.SortOrder > 0 {
			sort = uint32(d.SortOrder)
		}

		builder := tx.Language.Create().
			SetLanguageCode(d.LanguageCode).
			SetLanguageName(d.LanguageName).
			SetNillableNativeName(strPtr(d.NativeName)).
			SetIsDefault(d.IsDefault).
			SetIsEnabled(d.IsEnabled).
			SetNillableSortOrder(&sort).
			SetNillableCreatedBy(&operatorID).
			SetCreatedAt(time.Now()).
			SetUpdatedAt(time.Now())
		if _, err := builder.Save(l.ctx); err != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("batch create language failed: %v", err)
			return xerr.ServerErrorMsg("batch create language failed")
		}
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}