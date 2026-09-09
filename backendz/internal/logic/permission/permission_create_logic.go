// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/permission"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionCreateLogic {
	return &PermissionCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionCreateLogic) PermissionCreate(req *types.CreatePermissionRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" || strings.TrimSpace(d.Code) == "" {
		return xerr.BadRequestMsg("permission name and code required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	status := permission.StatusOn
	if d.Status != "" {
		status = permission.Status(d.Status)
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	create := tx.Permission.Create().
		SetName(d.Name).
		SetCode(d.Code).
		SetStatus(status).
		SetNillableDescription(nilStr(d.Description)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())
	if d.GroupId > 0 {
		create = create.SetNillableGroupID(u32Ptr(uint32(d.GroupId)))
	}

	created, cerr := create.Save(l.ctx)
	if cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create permission failed: %v", cerr)
		return xerr.ServerErrorMsg("create permission failed")
	}

	// 关联菜单
	for _, mid := range d.MenuIds {
		if mid <= 0 {
			continue
		}
		if _, perr := tx.PermissionMenu.Create().
			SetPermissionID(created.ID).
			SetMenuID(uint32(mid)).
			SetCreatedAt(time.Now()).
			SetUpdatedAt(time.Now()).
			Save(l.ctx); perr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("link permission menu failed: perm=%d menu=%d err=%v", created.ID, mid, perr)
			return xerr.ServerErrorMsg("link permission menu failed")
		}
	}

	// 关联 API
	for _, aid := range d.ApiIds {
		if aid <= 0 {
			continue
		}
		if _, perr := tx.PermissionApi.Create().
			SetPermissionID(created.ID).
			SetAPIID(uint32(aid)).
			SetCreatedAt(time.Now()).
			SetUpdatedAt(time.Now()).
			Save(l.ctx); perr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("link permission api failed: perm=%d api=%d err=%v", created.ID, aid, perr)
			return xerr.ServerErrorMsg("link permission api failed")
		}
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}

func nilStr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

func u32Ptr(v uint32) *uint32 {
	return &v
}
