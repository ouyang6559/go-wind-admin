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

type UserGetByIdLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserGetByIdLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserGetByIdLogic {
	return &UserGetByIdLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserGetByIdLogic) UserGetById(req *types.UserGetByIdReq) (resp *types.User, err error) {
	e, err := l.svcCtx.Ent.User.Query().
		Where(genuser.IDEQ(uint32(req.Id)), genuser.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("user not found")
		}
		logx.WithContext(l.ctx).Errorf("get user by id failed: %v", err)
		return nil, err
	}
	return UserToType(l.ctx, l.svcCtx.Ent, e)
}
