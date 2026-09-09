// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package position

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/position"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PositionCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPositionCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PositionCreateLogic {
	return &PositionCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PositionCreateLogic) PositionCreate(req *types.CreatePositionRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" {
		return xerr.BadRequestMsg("position name required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	posStatus := position.StatusOn
	if d.Status != "" {
		posStatus = position.Status(d.Status)
	}
	posType := position.TypeRegular
	if d.Type != "" {
		posType = position.Type(d.Type)
	}

	var sort uint32
	if d.SortOrder > 0 {
		sort = uint32(d.SortOrder)
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	b := tx.Position.Create().
		SetName(d.Name).
		SetCode(d.Code).
		SetNillableSortOrder(&sort).
		SetStatus(posStatus).
		SetType(posType).
		SetNillableDescription(strPtr(d.Description)).
		SetNillableRemark(strPtr(d.Remark)).
		SetNillableJobFamily(strPtr(d.JobFamily)).
		SetNillableJobGrade(strPtr(d.JobGrade)).
		SetNillableLevel(i32Ptr(d.Level)).
		SetNillableHeadcount(uiPtr(d.Headcount)).
		SetIsKeyPosition(d.IsKeyPosition).
		SetOrgUnitID(uint32(d.OrgUnitId)).
		SetNillableReportsToPositionID(uiPtr(d.ReportsToPositionId)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())

	if _, cerr := b.Save(l.ctx); cerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("create position failed: %v", cerr)
		return xerr.ServerErrorMsg("create position failed")
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}
