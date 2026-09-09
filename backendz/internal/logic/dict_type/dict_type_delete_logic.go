// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_type

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/dicttype"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictTypeDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictTypeDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictTypeDeleteLogic {
	return &DictTypeDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictTypeDeleteLogic) DictTypeDelete(req *types.DictTypeDeleteReq) error {
	for _, id := range req.Ids {
		if id <= 0 {
			continue
		}
		existing, err := l.svcCtx.Ent.DictType.Query().
			Where(dicttype.IDEQ(uint32(id)), dicttype.DeletedAtIsNil()).
			Only(l.ctx)
		if err != nil {
			if gen.IsNotFound(err) {
				continue
			}
			logx.WithContext(l.ctx).Errorf("get dict type for delete failed: %v", err)
			return xerr.ServerErrorMsg("get dict type for delete failed")
		}

		if err := l.svcCtx.Ent.DictType.UpdateOneID(existing.ID).
			SetDeletedAt(time.Now()).
			Exec(l.ctx); err != nil {
			logx.WithContext(l.ctx).Errorf("soft delete dict type failed: %v", err)
			return xerr.ServerErrorMsg("soft delete dict type failed")
		}
	}

	return nil
}