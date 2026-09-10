// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserListLogic {
	return &UserListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserListLogic) UserList(req *types.PageRequest) (resp *types.ListUserResponse, err error) {
	q := l.svcCtx.Ent.User.Query().Where(genuser.DeletedAtIsNil())

	// 搜索：按 用户名/昵称/真实姓名 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			genuser.Or(
				genuser.UsernameContainsFold(k),
				genuser.NicknameContainsFold(k),
				genuser.RealnameContainsFold(k),
			),
		)
	}

	q = q.Order(genuser.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count users failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list users failed: %v", err)
		return nil, err
	}

	items := make([]types.User, 0, len(rows))
	for _, r := range rows {
		item, cerr := UserToType(l.ctx, l.svcCtx.Ent, r)
		if cerr != nil {
			logx.WithContext(l.ctx).Errorf("convert user failed: %v", cerr)
			return nil, cerr
		}
		items = append(items, *item)
	}

	return &types.ListUserResponse{
		Items: items,
		Total: int64(total),
	}, nil
}
