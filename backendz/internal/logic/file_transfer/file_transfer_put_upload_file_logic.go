// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package file_transfer

import (
	"context"
	"path/filepath"
	"strings"
	"time"

	"github.com/google/uuid"

	"go-wind-admin/backendz/internal/ent/gen/file"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type FileTransferPutUploadFileLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewFileTransferPutUploadFileLogic(ctx context.Context, svcCtx *svc.ServiceContext) *FileTransferPutUploadFileLogic {
	return &FileTransferPutUploadFileLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *FileTransferPutUploadFileLogic) FileTransferPutUploadFile(req *types.UploadFileRequest) (resp *types.UploadFileResponse, err error) {
	if req.SourceFileName == "" {
		return nil, xerr.BadRequestMsg("sourceFileName required")
	}
	content, err := decodeFileContent(req.File)
	if err != nil {
		return nil, xerr.BadRequestMsg("invalid file content")
	}

	ext := strings.TrimPrefix(strings.ToLower(filepath.Ext(req.SourceFileName)), ".")
	objectName := uuid.NewString()
	if ext != "" {
		objectName = objectName + "." + ext
	}

	relPath, err := storeLocalFile(req.StorageObject.FileDirectory, objectName, content)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("store uploaded file failed: %v", err)
		return nil, xerr.ServerErrorMsg("store uploaded file failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	if err := l.svcCtx.Ent.File.Create().
		SetProvider(file.ProviderLocal).
		SetBucketName("local").
		SetFileDirectory(req.StorageObject.FileDirectory).
		SetFileGUID(uuid.NewString()).
		SetSaveFileName(objectName).
		SetFileName(req.SourceFileName).
		SetNillableExtension(strPtrOrNil(ext)).
		SetSize(uint64(len(content))).
		SetLinkURL(relPath).
		SetContentHash(contentHash(content)).
		SetCreatedBy(operatorID).
		SetCreatedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("record uploaded file metadata failed: %v", err)
		return nil, xerr.ServerErrorMsg("record uploaded file metadata failed")
	}

	return &types.UploadFileResponse{
		ObjectName: relPath,
	}, nil
}