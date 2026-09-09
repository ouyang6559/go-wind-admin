// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_entry

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/dictentry"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictEntryDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictEntryDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictEntryDeleteLogic {
	return &DictEntryDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictEntryDeleteLogic) DictEntryDelete(req *types.DictEntryDeleteReq) error {
	for _, id := range req.Ids {
		if id <= 0 {
			continue
		}
		existing, err := l.svcCtx.Ent.DictEntry.Query().
			Where(dictentry.IDEQ(uint32(id)), dictentry.DeletedAtIsNil()).
			Only(l.ctx)
		if err != nil {
			if gen.IsNotFound(err) {
				continue
			}
			logx.WithContext(l.ctx).Errorf("get dict entry for delete failed: %v", err)
			return xerr.ServerErrorMsg("get dict entry for delete failed")
		}

		if err := l.svcCtx.Ent.DictEntry.UpdateOneID(existing.ID).
			SetDeletedAt(time.Now()).
			Exec(l.ctx); err != nil {
			logx.WithContext(l.ctx).Errorf("soft delete dict entry failed: %v", err)
			return xerr.ServerErrorMsg("soft delete dict entry failed")
		}
	}

	return nil
}