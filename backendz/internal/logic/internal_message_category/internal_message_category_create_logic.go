// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message_category

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

type InternalMessageCategoryCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageCategoryCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageCategoryCreateLogic {
	return &InternalMessageCategoryCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageCategoryCreateLogic) InternalMessageCategoryCreate(req *types.CreateInternalMessageCategoryRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" {
		return xerr.BadRequestMsg("category name required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	created, err := l.svcCtx.Ent.InternalMessageCategory.Create().
		SetName(d.Name).
		SetNillableCode(strPtr(d.Code)).
		SetNillableIconURL(strPtr(d.IconUrl)).
		SetNillableSortOrder(uint32Ptr(d.SortOrder)).
		SetNillableIsEnabled(boolPtr(d.IsEnabled)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now()).
		Save(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("create internal message category failed: %v", err)
		return xerr.ServerErrorMsg("create internal message category failed")
	}
	_ = created

	return nil
}

func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

func uint32Ptr(v int64) *uint32 {
	if v <= 0 {
		return nil
	}
	u := uint32(v)
	return &u
}

func boolPtr(v bool) *bool {
	if !v {
		return nil
	}
	return &v
}