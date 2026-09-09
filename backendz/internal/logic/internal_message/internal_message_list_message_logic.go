// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/internalmessage"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageListMessageLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageListMessageLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageListMessageLogic {
	return &InternalMessageListMessageLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageListMessageLogic) InternalMessageListMessage(req *types.PageRequest) (resp *types.ListInternalMessageResponse, err error) {
	q := l.svcCtx.Ent.InternalMessage.Query().Where(internalmessage.DeletedAtIsNil())

	// 搜索：按标题模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(internalmessage.TitleContainsFold(k))
	}

	// 排序：创建时间倒序
	q = q.Order(internalmessage.ByCreatedAt(sql.OrderDesc()), internalmessage.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count internal messages failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list internal messages failed: %v", err)
		return nil, err
	}

	items := make([]types.InternalMessage, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(l.ctx, l.svcCtx.Ent, r))
	}

	return &types.ListInternalMessageResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}