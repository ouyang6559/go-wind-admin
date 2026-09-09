// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package file

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/file"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type FileListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewFileListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *FileListLogic {
	return &FileListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *FileListLogic) FileList(req *types.PageRequest) (resp *types.ListFileResponse, err error) {
	q := l.svcCtx.Ent.File.Query().Where(file.DeletedAtIsNil())

	// 搜索：按原始文件名或扩展名模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			file.Or(
				file.FileNameContainsFold(k),
				file.ExtensionContainsFold(k),
			),
		)
	}

	// 排序：默认按 id 降序
	q = q.Order(file.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count files failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list files failed: %v", err)
		return nil, err
	}

	items := make([]types.File, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListFileResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}