// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message_recipient

import (
	"context"
	"strconv"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/internalmessagerecipient"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageRecipientListUserInboxLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageRecipientListUserInboxLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageRecipientListUserInboxLogic {
	return &InternalMessageRecipientListUserInboxLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageRecipientListUserInboxLogic) InternalMessageRecipientListUserInbox(req *types.PageRequest) (resp *types.ListUserInboxResponse, err error) {
	var userID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		userID = c.UserID
	}
	if userID == 0 {
		return nil, xerr.ForbiddenMsg("user context required")
	}

	q := l.svcCtx.Ent.InternalMessageRecipient.Query().
		Where(internalmessagerecipient.RecipientUserIDEQ(userID), internalmessagerecipient.DeletedAtIsNil())

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count user inbox failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Order(internalmessagerecipient.ByID(sql.OrderDesc())).Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list user inbox failed: %v", err)
		return nil, err
	}

	items := make([]types.InternalMessageRecipient, 0, len(rows))
	for _, r := range rows {
		items = append(items, types.InternalMessageRecipient{
			Id:              int64(r.ID),
			MessageId:       std.Int64(r.MessageID),
			RecipientUserId: std.Int64(r.RecipientUserID),
			Status:          stringVal(r.Status),
			ReceivedAt:      std.TimeStr(r.ReceivedAt),
			ReadAt:          std.TimeStr(r.ReadAt),
			TenantId:        std.Int64(r.TenantID),
			TenantName:      tenantNameOf(l.ctx, l.svcCtx.Ent, r.TenantID),
			CreatedBy:       std.Int64(nil),
			UpdatedBy:       std.Int64(nil),
			DeletedBy:       std.Int64(nil),
			CreatedAt:       std.TimeStr(r.CreatedAt),
			UpdatedAt:       std.TimeStr(r.UpdatedAt),
			DeletedAt:       std.TimeStr(r.DeletedAt),
		})
	}

	// 批量回填消息标题/内容
	fillTitleContent(l.ctx, l.svcCtx.Ent, items)

	return &types.ListUserInboxResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}