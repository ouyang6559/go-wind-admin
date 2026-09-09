// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package file

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/file"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type FileUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewFileUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *FileUpdateLogic {
	return &FileUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *FileUpdateLogic) FileUpdate(req *types.UpdateFileRequest) error {
	if req.Id <= 0 {
		return xerr.BadRequestMsg("id required")
	}
	d := req.Data
	if _, err := l.svcCtx.Ent.File.Query().
		Where(file.IDEQ(uint32(req.Id)), file.DeletedAtIsNil()).
		Only(l.ctx); err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("file not found")
		}
		logx.WithContext(l.ctx).Errorf("get file for update failed: %v", err)
		return xerr.ServerErrorMsg("get file for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.File.UpdateOneID(uint32(req.Id)).
		SetNillableProvider(providerPtr(d.Provider)).
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
		SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())

	if _, err := upd.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("update file failed: %v", err)
		return xerr.ServerErrorMsg("update file failed")
	}

	return nil
}

// providerPtr 将字符串 provider 映射为 *file.Provider；空串返回 nil。
func providerPtr(s string) *file.Provider {
	if s == "" {
		return nil
	}
	p := providerEnum(s)
	return &p
}