// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package file

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/file"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type FileCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewFileCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *FileCreateLogic {
	return &FileCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *FileCreateLogic) FileCreate(req *types.CreateFileRequest) error {
	d := req.Data

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	b := l.svcCtx.Ent.File.Create().
		SetProvider(providerEnum(d.Provider)).
		SetNillableBucketName(strPtr(d.BucketName)).
		SetNillableFileDirectory(strPtr(d.FileDirectory)).
		SetNillableFileGUID(strPtr(d.FileGuid)).
		SetNillableSaveFileName(strPtr(d.SaveFileName)).
		SetNillableFileName(strPtr(d.FileName)).
		SetNillableExtension(strPtr(d.Extension)).
		SetNillableSize(nillableSize(d.Size)).
		SetNillableSizeFormat(strPtr(d.SizeFormat)).
		SetNillableLinkURL(strPtr(d.LinkUrl)).
		SetNillableContentHash(strPtr(d.ContentHash)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())

	if d.Id > 0 {
		b.SetID(uint32(d.Id))
	}

	if err := b.Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("create file failed: %v", err)
		return xerr.ServerErrorMsg("create file failed")
	}

	return nil
}

// providerEnum 将字符串 provider 映射为文件枚举；空串使用默认 MINIO。
func providerEnum(s string) file.Provider {
	switch file.Provider(s) {
	case file.ProviderMinIO, file.ProviderAliyun, file.ProviderQiniu, file.ProviderTencent,
		file.ProviderAWS, file.ProviderGoogle, file.ProviderAzure, file.ProviderBaidu,
		file.ProviderHuawei, file.ProviderLocal, file.ProviderUnknown:
		return file.Provider(s)
	default:
		return file.ProviderMinIO
	}
}