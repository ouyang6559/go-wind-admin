// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package notification_channel

import (
	"context"
	"strconv"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen/notificationchannel"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type NotificationChannelListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewNotificationChannelListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *NotificationChannelListLogic {
	return &NotificationChannelListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *NotificationChannelListLogic) NotificationChannelList(req *types.PageRequest) (resp *types.ListNotificationChannelResponse, err error) {
	q := l.svcCtx.Ent.NotificationChannel.Query().Where(notificationchannel.DeletedAtIsNil())

	// 搜索：按 name 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(notificationchannel.NameContainsFold(k))
	}

	// 排序：ID 升序
	q = q.Order(notificationchannel.ByID())

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count notification channels failed: %v", err)
		return nil, xerr.ServerErrorMsg("count notification channels failed")
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list notification channels failed: %v", err)
		return nil, xerr.ServerErrorMsg("list notification channels failed")
	}

	items := make([]types.NotificationChannel, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListNotificationChannelResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}
