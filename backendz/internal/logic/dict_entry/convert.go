package dict_entry

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/dictentry"
	"go-wind-admin/backendz/internal/ent/gen/dictentryi18n"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.DictEntry（需预先加载 dict_type 边以取 TypeId）。
func toType(e *gen.DictEntry) *types.DictEntry {
	out := &types.DictEntry{
		Id:           int64(e.ID),
		EntryValue:   std.Str(e.EntryValue),
		IsEnabled:    std.Bool(e.IsEnabled),
		SortOrder:    std.Int64(e.SortOrder),
		TenantId:     std.Int64(e.TenantID),
		CreatedBy:    std.Int64(e.CreatedBy),
		UpdatedBy:    std.Int64(e.UpdatedBy),
		DeletedBy:    std.Int64(e.DeletedBy),
		CreatedAt:    std.TimeStr(e.CreatedAt),
		UpdatedAt:    std.TimeStr(e.UpdatedAt),
		DeletedAt:    std.TimeStr(e.DeletedAt),
	}
	if e.NumericValue != nil {
		out.NumericValue = int64(*e.NumericValue)
	}
	if e.Edges.DictType != nil {
		out.TypeId = int64(e.Edges.DictType.ID)
	}
	return out
}

// i18nOf 查询指定字典项 ID 的多语言数据并转换为返回结构。
func i18nOf(ctx context.Context, client *gen.Client, entryID uint32) map[string]interface{} {
	rows, err := client.DictEntryI18n.Query().
		Where(dictentryi18n.HasDictEntryWith(dictentry.IDEQ(entryID)), dictentryi18n.DeletedAtIsNil()).
		All(ctx)
	if err != nil {
		return nil
	}

	out := make(map[string]interface{}, len(rows))
	for _, r := range rows {
		lc := std.Str(r.LanguageCode)
		if lc == "" {
			continue
		}
		out[lc] = map[string]interface{}{
			"entryLabel":   std.Str(r.EntryLabel),
			"description":  std.Str(r.Description),
			"languageCode": lc,
		}
	}
	return out
}

// replaceI18n 在事务内先清理再重建指定字典项的多语言数据。
func replaceI18n(ctx context.Context, tx *gen.Tx, entryID, operatorID uint32, i18n map[string]interface{}) error {
	if _, err := tx.DictEntryI18n.Delete().
		Where(dictentryi18n.HasDictEntryWith(dictentry.IDEQ(entryID))).
		Exec(ctx); err != nil {
		return err
	}

	if len(i18n) == 0 {
		return nil
	}

	now := time.Now()
	for langCode, v := range i18n {
		label, desc := parseI18nItem(v)
		builder := tx.DictEntryI18n.Create().
			SetLanguageCode(langCode).
			SetDictEntryID(entryID).
			SetNillableCreatedBy(&operatorID).
			SetCreatedAt(now).
			SetUpdatedAt(now)
		if label != "" {
			builder.SetEntryLabel(label)
		}
		if desc != "" {
			builder.SetDescription(desc)
		}
		if _, err := builder.Save(ctx); err != nil {
			return err
		}
	}

	return nil
}

// parseI18nItem 兼容 map[string]interface{} 与 types.DictEntryI18n 两种入参。
func parseI18nItem(v interface{}) (label, desc string) {
	switch t := v.(type) {
	case types.DictEntryI18n:
		return t.EntryLabel, t.Description
	case map[string]interface{}:
		if s, ok := t["entryLabel"].(string); ok {
			label = s
		}
		if s, ok := t["description"].(string); ok {
			desc = s
		}
	}
	return
}