// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package language

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/language"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LanguageGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLanguageGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LanguageGetLogic {
	return &LanguageGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LanguageGetLogic) LanguageGet(req *types.LanguageGetReq) (resp *types.Language, err error) {
	q := l.svcCtx.Ent.Language.Query().Where(language.DeletedAtIsNil())
	if req.Id > 0 {
		q = q.Where(language.IDEQ(uint32(req.Id)))
	} else if req.Code != "" {
		q = q.Where(language.LanguageCodeEQ(req.Code))
	} else {
		return nil, xerr.BadRequestMsg("id or code required")
	}

	e, err := q.Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("language not found")
		}
		logx.WithContext(l.ctx).Errorf("get language failed: %v", err)
		return nil, err
	}

	return toType(e), nil
}