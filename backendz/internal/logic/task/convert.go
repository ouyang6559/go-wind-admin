package task

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/schema"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 任务实体转为前端类型。
func toType(e *gen.Task) *types.Task {
	return &types.Task{
		Id:          int64(e.ID),
		Type:        stringVal(e.Type),
		TypeName:    std.Str(e.TypeName),
		TaskPayload: std.Str(e.TaskPayload),
		CronSpec:    std.Str(e.CronSpec),
		TaskOptions: typeOptFromSchema(e.TaskOptions),
		Enable:      std.Bool(e.Enable),
		Remark:      std.Str(e.Remark),
		TenantId:    std.Int64(e.TenantID),
		CreatedBy:   std.Int64(e.CreatedBy),
		UpdatedBy:   std.Int64(e.UpdatedBy),
		DeletedBy:   std.Int64(e.DeletedBy),
		CreatedAt:   std.TimeStr(e.CreatedAt),
		UpdatedAt:   std.TimeStr(e.UpdatedAt),
		DeletedAt:   std.TimeStr(e.DeletedAt),
	}
}

// schemaOptFromType 将前端 TaskOption 转为 ent JSON 结构体。
func schemaOptFromType(o types.TaskOption) *schema.TaskOption {
	var maxRetry *uint32
	if o.MaxRetry > 0 {
		v := uint32(o.MaxRetry)
		maxRetry = &v
	}
	return &schema.TaskOption{
		MaxRetry:  maxRetry,
		Timeout:   o.Timeout,
		Deadline:  o.Deadline,
		ProcessIn: o.ProcessIn,
		ProcessAt: o.ProcessAt,
		UniqueTTL: o.UniqueTTL,
		Retention: o.Retention,
		Group:     o.Group,
		TaskID:    o.TaskID,
	}
}

// typeOptFromSchema 将 ent JSON 结构体转回前端 TaskOption。
func typeOptFromSchema(o *schema.TaskOption) types.TaskOption {
	if o == nil {
		return types.TaskOption{}
	}
	var maxRetry int64
	if o.MaxRetry != nil {
		maxRetry = int64(*o.MaxRetry)
	}
	return types.TaskOption{
		MaxRetry:  maxRetry,
		Timeout:   o.Timeout,
		Deadline:  o.Deadline,
		ProcessIn: o.ProcessIn,
		ProcessAt: o.ProcessAt,
		UniqueTTL: o.UniqueTTL,
		Retention: o.Retention,
		Group:     o.Group,
		TaskID:    o.TaskID,
	}
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}