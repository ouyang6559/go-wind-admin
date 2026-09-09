// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package file_transfer

import (
	"context"
	"path"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/file"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type FileTransferDownloadFileLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewFileTransferDownloadFileLogic(ctx context.Context, svcCtx *svc.ServiceContext) *FileTransferDownloadFileLogic {
	return &FileTransferDownloadFileLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *FileTransferDownloadFileLogic) FileTransferDownloadFile(req *types.FileTransferDownloadFileReq) (resp *types.DownloadFileResponse, err error) {
	// 仅支持按 fileId 从本地临时目录回读内容；downloadUrl 直传在此最小实现中返回不支持。
	if req.FileId <= 0 {
		return nil, xerr.BadRequestMsg("fileId required")
	}

	e, err := l.svcCtx.Ent.File.Query().
		Where(file.IDEQ(uint32(req.FileId)), file.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("file not found")
		}
		logx.WithContext(l.ctx).Errorf("get file for download failed: %v", err)
		return nil, xerr.ServerErrorMsg("get file for download failed")
	}

	// 重建相对存储路径：目录 + "/" + 实际存储文件名。
	var relPath string
	if e.LinkURL != nil && *e.LinkURL != "" {
		relPath = *e.LinkURL
	} else {
		relPath = joinObjectDir(fileStr(e.FileDirectory), fileStr(e.SaveFileName))
	}

	fileData, err := readLocalFile(relPath)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("read local file for download failed: rel=%s err=%v", relPath, err)
		return nil, xerr.NotFoundMsg("file content not found")
	}

	return &types.DownloadFileResponse{
		File:           fileData,
		SourceFileName: fileStr(e.FileName),
		Mime:           fileStr(e.Extension),
		Size:           fileStr(e.SizeFormat),
		StoragePath:    relPath,
	}, nil
}

func fileStr(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

// joinObjectDir 拼接目录与文件名，目录为空时不加前导斜杠。
func joinObjectDir(directory, fileName string) string {
	directory = path.Clean("/" + directory)
	directory = directory[1:]
	if directory == "" || directory == "." {
		return fileName
	}
	return directory + "/" + fileName
}