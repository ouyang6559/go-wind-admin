// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package role

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type RoleCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewRoleCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *RoleCreateLogic {
	return &RoleCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *RoleCreateLogic) RoleCreate(req *types.CreateRoleRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" || strings.TrimSpace(d.Code) == "" {
		return xerr.BadRequestMsg("role name and code required")
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

	roleStatus := role.StatusOn
	if d.Status != "" {
		roleStatus = role.Status(d.Status)
	}
	roleType := role.TypeTenant
	if d.Type != "" {
		roleType = role.Type(d.Type)
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	created, cerr := tx.Role.Create().
		SetName(d.Name).
		SetCode(d.Code).
		SetNillableSortOrder(&sort).
		SetStatus(roleStatus).
		SetType(roleType).
		SetNillableDescription(strPtr(d.Description)).
		SetIsProtected(d.IsProtected).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now()).
		Save(l.ctx)
	if cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create role failed: %v", cerr)
		return xerr.ServerErrorMsg("create role failed")
	}

	// 关联权限
	for _, pid := range d.Permissions {
		if pid <= 0 {
			continue
		}
		_, perr := tx.RolePermission.Create().
			SetRoleID(created.ID).
			SetPermissionID(uint32(pid)).
			SetCreatedAt(time.Now()).
			SetUpdatedAt(time.Now()).
			Save(l.ctx)
		if perr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("link role permission failed: roleID=%d permissionID=%d err=%v", created.ID, pid, perr)
			return xerr.ServerErrorMsg("link role permission failed")
		}
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}

func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}
