// Package std 提供跨 logic 复用的通用工具：分页、时间格式化、指针取值等。
package std

import (
	"strconv"
	"time"
)

// Paginate 计算分页 offset/limit。page 从 1 开始，pageSize<=0 时给默认 20。
func Paginate(page, pageSize int64) (offset, limit int) {
	if pageSize <= 0 {
		pageSize = 20
	}
	if pageSize > 1000 {
		pageSize = 1000
	}
	if page < 1 {
		page = 1
	}
	return int((page - 1) * pageSize), int(pageSize)
}

// TimeStr 返回 time.Time 的 RFC3339 字符串；nil 返回空串。
func TimeStr(t *time.Time) string {
	if t == nil {
		return ""
	}
	return t.Format(time.RFC3339)
}

// Str 解引用 *string；nil 返回空串。
func Str(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

// Bool 解引用 *bool；nil 返回 false。
func Bool(b *bool) bool {
	if b == nil {
		return false
	}
	return *b
}

// Int64 解引用 *int64/*uint32 为 int64；nil 返回 0。
func Int64(v any) int64 {
	switch x := v.(type) {
	case *int64:
		if x == nil {
			return 0
		}
		return *x
	case *uint32:
		if x == nil {
			return 0
		}
		return int64(*x)
	case *uint64:
		if x == nil {
			return 0
		}
		return int64(*x)
	case *int:
		if x == nil {
			return 0
		}
		return int64(*x)
	case *string:
		return Int(strconv.ParseInt(Str(x), 10, 64))
	case int64:
		return x
	case uint32:
		return int64(x)
	}
	return 0
}

// Int 解析 int64，error 时返回 0。
func Int(v int64, err error) int64 {
	if err != nil {
		return 0
	}
	return v
}

// PtrBool 就地解布尔指针并断言目标值（方便链式）。
func PtrBool(b *bool) bool {
	return Bool(b)
}