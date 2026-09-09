// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package file

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/file"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type FileDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewFileDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *FileDeleteLogic {
	return &FileDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *FileDeleteLogic) FileDelete(req *types.FileDeleteReq) error {
	if _, err := l.svcCtx.Ent.File.Query().
		Where(file.IDEQ(uint32(req.Id)), file.DeletedAtIsNil()).
		Only(l.ctx); err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("file not found")
		}
		logx.WithContext(l.ctx).Errorf("get file for delete failed: %v", err)
		return xerr.ServerErrorMsg("get file for delete failed")
	}

	if err := l.svcCtx.Ent.File.UpdateOneID(uint32(req.Id)).
		SetDeletedAt(time.Now()).Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete file failed: %v", err)
		return xerr.ServerErrorMsg("soft delete file failed")
	}

	return nil
}