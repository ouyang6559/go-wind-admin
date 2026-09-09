// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user

import (
	"context"

	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserUserExistsLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserUserExistsLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserUserExistsLogic {
	return &UserUserExistsLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserUserExistsLogic) UserUserExists(req *types.UserUserExistsReq) (resp *types.UserExistsResponse, err error) {
	q := l.svcCtx.Ent.User.Query().Where(genuser.DeletedAtIsNil())
	if req.Id > 0 && req.Username != "" {
		q = q.Where(genuser.Or(genuser.IDEQ(uint32(req.Id)), genuser.UsernameEQ(req.Username)))
	} else if req.Id > 0 {
		q = q.Where(genuser.IDEQ(uint32(req.Id)))
	} else if req.Username != "" {
		q = q.Where(genuser.UsernameEQ(req.Username))
	} else {
		return &types.UserExistsResponse{Exist: false}, nil
	}

	count, cerr := q.Count(l.ctx)
	if cerr != nil {
		logx.WithContext(l.ctx).Errorf("check user exists failed: %v", cerr)
		return nil, cerr
	}
	return &types.UserExistsResponse{Exist: count > 0}, nil
}
