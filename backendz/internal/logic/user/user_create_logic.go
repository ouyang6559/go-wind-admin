// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user

import (
	"context"
	"time"

	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/ent/gen/userrole"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserCreateLogic {
	return &UserCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserCreateLogic) UserCreate(req *types.CreateUserRequest) error {
	username := req.Data.Username
	if username == "" {
		return xerr.BadRequestMsg("username is required")
	}

	var operatorID, operatorTenant uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
		operatorTenant = c.TenantID
	}

	tid := operatorTenant
	if req.Data.TenantId > 0 {
		tid = uint32(req.Data.TenantId)
	}

	plain := req.Password
	if plain == "" {
		plain = "123456"
	}
	hash, herr := password.Hash(plain)
	if herr != nil {
		logx.WithContext(l.ctx).Errorf("hash password failed: %v", herr)
		return xerr.ServerErrorMsg("hash password failed")
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	created, cerr := tx.User.Create().
		SetTenantID(tid).
		SetUsername(username).
		SetStatus(genuser.StatusNormal).
		SetNillableCreatedBy(&operatorID).
		SetNillableNickname(strPtr(req.Data.Nickname)).
		SetNillableRealname(strPtr(req.Data.Realname)).
		SetNillableEmail(strPtr(req.Data.Email)).
		SetNillableMobile(strPtr(req.Data.Mobile)).
		SetNillableAvatar(strPtr(req.Data.Avatar)).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now()).
		Save(l.ctx)
	if cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create user failed: %v", cerr)
		return xerr.ServerErrorMsg("create user failed")
	}

	if _, c2err := tx.UserCredential.Create().
		SetTenantID(tid).
		SetUserID(created.ID).
		SetIdentityType(usercredential.IdentityTypeUsername).
		SetIdentifier(username).
		SetCredentialType(usercredential.CredentialTypePasswordHash).
		SetCredential(hash).
		SetIsPrimary(true).
		SetStatus(usercredential.StatusEnabled).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now()).
		Save(l.ctx); c2err != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create user credential failed: %v", c2err)
		return xerr.ServerErrorMsg("create user credential failed")
	}

	// 绑定角色（可在多个角色间复用事务）
	now := time.Now()
	for _, rid := range req.Data.RoleIds {
		if rid <= 0 {
			continue
		}
		if _, rerr := tx.UserRole.Create().
			SetTenantID(tid).
			SetUserID(created.ID).
			SetRoleID(uint32(rid)).
			SetStatus(userrole.StatusActive).
			SetIsPrimary(true).
			SetNillableAssignedBy(&operatorID).
			SetNillableAssignedAt(&now).
			SetCreatedAt(now).
			SetUpdatedAt(now).
			Save(l.ctx); rerr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("bind user role failed: %v", rerr)
			return xerr.ServerErrorMsg("bind user role failed")
		}
	}

	if cerr := tx.Commit(); cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", cerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}
	return nil
}
