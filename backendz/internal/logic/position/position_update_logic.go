// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package position

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/position"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PositionUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPositionUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PositionUpdateLogic {
	return &PositionUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PositionUpdateLogic) PositionUpdate(req *types.UpdatePositionRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Position.Query().
		Where(position.IDEQ(uint32(req.Id)), position.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("position not found")
		}
		logx.WithContext(l.ctx).Errorf("get position for update failed: %v", err)
		return xerr.ServerErrorMsg("get position for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.Position.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if d.Code != "" {
		upd.SetCode(d.Code)
	}
	if d.Remark != "" {
		upd.SetRemark(d.Remark)
	}
	if d.Description != "" {
		upd.SetDescription(d.Description)
	}
	if d.Status != "" {
		upd.SetStatus(position.Status(d.Status))
	}
	if d.Type != "" {
		upd.SetType(position.Type(d.Type))
	}
	if d.JobFamily != "" {
		upd.SetJobFamily(d.JobFamily)
	}
	if d.JobGrade != "" {
		upd.SetJobGrade(d.JobGrade)
	}
	if d.Level != 0 {
		upd.SetLevel(int32(d.Level))
	}
	if d.Headcount > 0 {
		upd.SetHeadcount(uint32(d.Headcount))
	}
	if d.SortOrder > 0 {
		upd.SetSortOrder(uint32(d.SortOrder))
	}
	if d.OrgUnitId > 0 {
		upd.SetOrgUnitID(uint32(d.OrgUnitId))
	}
	if d.ReportsToPositionId > 0 {
		upd.SetReportsToPositionID(uint32(d.ReportsToPositionId))
	}
	if d.IsKeyPosition {
		upd.SetIsKeyPosition(true)
	}

	if _, err := upd.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("update position failed: %v", err)
		return xerr.ServerErrorMsg("update position failed")
	}

	return nil
}
