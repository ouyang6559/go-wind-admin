// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserDeleteByIdLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserDeleteByIdLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserDeleteByIdLogic {
	return &UserDeleteByIdLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserDeleteByIdLogic) UserDeleteById(req *types.UserDeleteByIdReq) error {
	u, err := l.svcCtx.Ent.User.Query().
		Where(genuser.IDEQ(uint32(req.Id)), genuser.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("user not found")
		}
		logx.WithContext(l.ctx).Errorf("get user for delete failed: %v", err)
		return xerr.ServerErrorMsg("get user failed")
	}
	return softDeleteUser(l.ctx, l.svcCtx, u)
}
