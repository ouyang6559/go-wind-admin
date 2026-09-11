package authentication

import (
	"context"

	"entgo.io/ent/privacy"

	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type AuthenticationRegisterUserLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAuthenticationRegisterUserLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AuthenticationRegisterUserLogic {
	return &AuthenticationRegisterUserLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *AuthenticationRegisterUserLogic) AuthenticationRegisterUser(req *types.RegisterUserRequest) (resp *types.RegisterUserResponse, err error) {
	if req.Username == "" || req.Password == "" {
		return nil, xerr.BadRequestMsg("username and password required")
	}

	// 注册归属平台租户（0）；传入 tenantCode 时校验租户存在且启用
	var tenantID uint32
	// （简化）暂不开放跨租户自助注册，统一归属平台。

	hash, herr := password.Hash(req.Password)
	if herr != nil {
		return nil, xerr.ServerErrorMsg("hash password failed")
	}

	// 事务内创建用户 + 凭证
	allowCtx := privacy.DecisionContext(l.ctx, privacy.Allow)
	tx, terr := l.svcCtx.Ent.Tx(allowCtx)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return nil, xerr.ServerErrorMsg("begin tx failed")
	}

	u, uerr := tx.User.Create().
		SetTenantID(tenantID).
		SetUsername(req.Username).
		SetStatus(user.StatusNormal).
		Save(allowCtx)
	if uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create user failed: %v", uerr)
		return nil, xerr.BadRequestMsg("create user failed")
	}

	_, cerr := tx.UserCredential.Create().
		SetTenantID(tenantID).
		SetUserID(u.ID).
		SetIdentityType(usercredential.IdentityTypeUsername).
		SetIdentifier(req.Username).
		SetCredentialType(usercredential.CredentialTypePasswordHash).
		SetCredential(hash).
		SetIsPrimary(true).
		SetStatus(usercredential.StatusEnabled).
		Save(allowCtx)
	if cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create credential failed: %v", cerr)
		return nil, xerr.BadRequestMsg("create credential failed")
	}

	if err := tx.Commit(); err != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", err)
		return nil, xerr.ServerErrorMsg("commit tx failed")
	}

	return &types.RegisterUserResponse{
		UserId: int64(u.ID),
	}, nil
}
