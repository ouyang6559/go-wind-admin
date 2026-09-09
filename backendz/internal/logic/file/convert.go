package file

import (
	"strconv"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/file"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent File 实体转换为 types.File。
func toType(e *gen.File) *types.File {
	return &types.File{
		Id:            int64(e.ID),
		Provider:      providerStr(e.Provider),
		BucketName:    std.Str(e.BucketName),
		FileDirectory: std.Str(e.FileDirectory),
		FileGuid:      std.Str(e.FileGUID),
		SaveFileName:  std.Str(e.SaveFileName),
		FileName:      std.Str(e.FileName),
		Extension:     std.Str(e.Extension),
		Size:          sizeStr(e.Size),
		SizeFormat:    std.Str(e.SizeFormat),
		LinkUrl:       std.Str(e.LinkURL),
		ContentHash:   std.Str(e.ContentHash),
		TenantId:      std.Int64(e.TenantID),
		CreatedBy:     std.Int64(e.CreatedBy),
		UpdatedBy:     std.Int64(e.UpdatedBy),
		DeletedBy:     std.Int64(e.DeletedBy),
		CreatedAt:     std.TimeStr(e.CreatedAt),
		UpdatedAt:     std.TimeStr(e.UpdatedAt),
		DeletedAt:     std.TimeStr(e.DeletedAt),
	}
}

func providerStr(p *file.Provider) string {
	if p == nil {
		return ""
	}
	return string(*p)
}

func sizeStr(s *uint64) string {
	if s == nil {
		return ""
	}
	return strconv.FormatUint(*s, 10)
}

// nillableSize 将字符串形式的 size 解析为 *uint64；空串或非法值返回 nil。
func nillableSize(s string) *uint64 {
	if s == "" {
		return nil
	}
	v, err := strconv.ParseUint(s, 10, 64)
	if err != nil {
		return nil
	}
	return &v
}

// strPtr 空串返回 nil，否则返回指针（用于 SetNillable）。
func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}