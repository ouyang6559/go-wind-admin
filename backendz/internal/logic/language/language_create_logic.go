// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package language

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

type LanguageCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLanguageCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LanguageCreateLogic {
	return &LanguageCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LanguageCreateLogic) LanguageCreate(req *types.CreateLanguageRequest) error {
	d := req.Data
	if strings.TrimSpace(d.LanguageCode) == "" || strings.TrimSpace(d.LanguageName) == "" {
		return xerr.BadRequestMsg("language code and name required")
	}

	return l.createOne(l.ctx, d)
}

func (l *LanguageCreateLogic) createOne(ctx context.Context, d types.Language) error {
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(ctx); ok {
		operatorID = c.UserID
	}

	var sort uint32
	if d.SortOrder > 0 {
		sort = uint32(d.SortOrder)
	}

	builder := l.svcCtx.Ent.Language.Create().
		SetLanguageCode(d.LanguageCode).
		SetLanguageName(d.LanguageName).
		SetNillableNativeName(strPtr(d.NativeName)).
		SetIsDefault(d.IsDefault).
		SetIsEnabled(d.IsEnabled).
		SetNillableSortOrder(&sort).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())

	if _, err := builder.Save(ctx); err != nil {
		logx.WithContext(ctx).Errorf("create language failed: %v", err)
		return xerr.ServerErrorMsg("create language failed")
	}

	return nil
}

func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}