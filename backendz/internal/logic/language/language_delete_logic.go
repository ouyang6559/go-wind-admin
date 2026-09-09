// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package language

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/language"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LanguageDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLanguageDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LanguageDeleteLogic {
	return &LanguageDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LanguageDeleteLogic) LanguageDelete(req *types.LanguageDeleteReq) error {
	if req.Id <= 0 {
		return xerr.BadRequestMsg("id required")
	}

	existing, err := l.svcCtx.Ent.Language.Query().
		Where(language.IDEQ(uint32(req.Id)), language.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("language not found")
		}
		logx.WithContext(l.ctx).Errorf("get language for delete failed: %v", err)
		return xerr.ServerErrorMsg("get language for delete failed")
	}

	if err := l.svcCtx.Ent.Language.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete language failed: %v", err)
		return xerr.ServerErrorMsg("soft delete language failed")
	}

	return nil
}