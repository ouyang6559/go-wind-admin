package oss

import (
	"github.com/minio/minio-go/v7"
)

// SetDownloadRange 设置下载范围
func SetDownloadRange(opts *minio.GetObjectOptions, start, end *int64) {
	if opts == nil {
		return
	}

	if start != nil && end != nil {
		_ = opts.SetRange(*start, *end)
	} else if start != nil {
		_ = opts.SetRange(*start, 0)
	} else if end != nil {
		_ = opts.SetRange(0, *end)
	}
}
