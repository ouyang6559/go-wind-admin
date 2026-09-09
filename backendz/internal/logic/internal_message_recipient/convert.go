package internal_message_recipient

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessage"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// fillTitleContent 一次 IN 查询批量回填消息标题与内容（收件箱高频轮询路径，避免 N+1）。
// 回填失败仅影响展示，不阻断整页返回。
func fillTitleContent(ctx context.Context, client *gen.Client, items []types.InternalMessageRecipient) {
	if len(items) == 0 {
		return
	}

	messageIDs := make([]uint32, 0, len(items))
	for i := range items {
		if items[i].MessageId > 0 {
			messageIDs = append(messageIDs, uint32(items[i].MessageId))
		}
	}
	if len(messageIDs) == 0 {
		return
	}

	messages, err := client.InternalMessage.Query().
		Where(internalmessage.IDIn(messageIDs...), internalmessage.DeletedAtIsNil()).
		Select(internalmessage.FieldID, internalmessage.FieldTitle, internalmessage.FieldContent).
		All(ctx)
	if err != nil {
		// best-effort：取不到消息本体只影响 title/content 展示。
		return
	}
	byID := make(map[uint32]*gen.InternalMessage, len(messages))
	for _, m := range messages {
		byID[m.ID] = m
	}
	for i := range items {
		if m, ok := byID[uint32(items[i].MessageId)]; ok {
			items[i].Title = std.Str(m.Title)
			items[i].Content = std.Str(m.Content)
		}
	}
}

// tenantNameOf 查询租户名；找不到或 err 时返回空串（不阻断）。
func tenantNameOf(ctx context.Context, client *gen.Client, tid *uint32) string {
	if tid == nil {
		return ""
	}
	t, err := client.Tenant.Query().
		Where(tenant.IDEQ(*tid), tenant.DeletedAtIsNil()).
		Select(tenant.FieldName).
		Only(ctx)
	if err != nil {
		return ""
	}
	return std.Str(t.Name)
}