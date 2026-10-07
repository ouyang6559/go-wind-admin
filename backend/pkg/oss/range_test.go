package oss

import (
	"testing"

	"github.com/minio/minio-go/v7"
	"github.com/stretchr/testify/require"
)

// TestSetDownloadRange 表驱动校验区间写入：
// (start,end) 双有 → bytes=N-M；仅 start → bytes=N-；仅 end → bytes=0-N（minio SetRange 语义）；
// 双无 → 不设置头；nil opts → 直接返回不 panic。
func TestSetDownloadRange(t *testing.T) {
	tests := []struct {
		name      string
		start     *int64
		end       *int64
		wantRange string
	}{
		{"both start and end", int64Ptr(5), int64Ptr(10), "bytes=5-10"},
		{"start only", int64Ptr(5), nil, "bytes=5-"},
		{"end only", nil, int64Ptr(-5), "bytes=-5"},
		{"end only positive", nil, int64Ptr(5), "bytes=0-5"},
		{"neither", nil, nil, ""},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			var opts minio.GetObjectOptions
			SetDownloadRange(&opts, tt.start, tt.end)
			require.Equal(t, tt.wantRange, opts.Header().Get("Range"),
				"Range header for start=%v end=%v", tt.start, tt.end)
		})
	}
}

// TestSetDownloadRange_NilOpts nil 选项对象必须直接返回且不 panic。
func TestSetDownloadRange_NilOpts(t *testing.T) {
	t.Parallel()
	require.NotPanics(t, func() {
		SetDownloadRange(nil, nil, nil)
	})
}

// int64Ptr 构造 int64 指针的测试辅助。
func int64Ptr(v int64) *int64 { return &v }
