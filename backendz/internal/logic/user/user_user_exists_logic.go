// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user

import (
	"context"

	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

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
	// 对齐 kratos UserExists 的 oneof 语义：id 与 username 必须且只能提供其一，否则视为非法查询。
	if (req.Id > 0) == (req.Username != "") {
		return &types.UserExistsResponse{Exist: false}, xerr.BadRequestMsg("invalid query by type")
	}

	q := l.svcCtx.Ent.User.Query().Where(genuser.DeletedAtIsNil())
	switch {
	case req.Id > 0:
		q = q.Where(genuser.IDEQ(uint32(req.Id)))
	default:
		// username 仅在 (tenant_id, username) 维度唯一；平台上下文(tid=0)下按 username 查存在性
		// 会跨租户泄露（任意租户有同名即 true）。与 kratos 一致，仅允许具名租户上下文(tid>0)查询。
		if !l.hasTenantContext() {
			return &types.UserExistsResponse{Exist: false}, xerr.BadRequestMsg("tenant scope required")
		}
		q = q.Where(genuser.UsernameEQ(req.Username))
	}

	count, cerr := q.Count(l.ctx)
	if cerr != nil {
		logx.WithContext(l.ctx).Errorf("check user exists failed: %v", cerr)
		return nil, cerr
	}
	return &types.UserExistsResponse{Exist: count > 0}, nil
}

// hasTenantContext 判断当前请求是否处于具名租户上下文（JWT 载荷中 tid>0）。
func (l *UserUserExistsLogic) hasTenantContext() bool {
	claims, ok := middleware.ClaimsFromContext(l.ctx)
	return ok && claims.TenantID > 0
}
