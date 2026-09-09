// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserUpdateLogic {
	return &UserUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserUpdateLogic) UserUpdate(req *types.UpdateUserRequest) error {
	existing, err := l.svcCtx.Ent.User.Query().
		Where(genuser.IDEQ(uint32(req.Id)), genuser.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("user not found")
		}
		logx.WithContext(l.ctx).Errorf("get user for update failed: %v", err)
		return xerr.ServerErrorMsg("get user for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.User.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	ApplyUserPatch(upd, req.Data)

	if _, err := upd.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("update user failed: %v", err)
		return xerr.ServerErrorMsg("update user failed")
	}

	// 设置密码则一并重置用户名凭证
	if req.Password != "" {
		if err := resetCredentialUsername(l.ctx, l.svcCtx, existing.Username, req.Password); err != nil {
			return err
		}
	}

	return nil
}
