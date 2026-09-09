// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package login_policy

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/loginpolicy"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LoginPolicyUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLoginPolicyUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LoginPolicyUpdateLogic {
	return &LoginPolicyUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LoginPolicyUpdateLogic) LoginPolicyUpdate(req *types.UpdateLoginPolicyRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.LoginPolicy.Query().
		Where(loginpolicy.IDEQ(uint32(req.Id)), loginpolicy.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("login policy not found")
		}
		logx.WithContext(l.ctx).Errorf("get login policy for update failed: %v", err)
		return xerr.ServerErrorMsg("get login policy for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.LoginPolicy.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.TargetId > 0 {
		upd.SetTargetID(uint32(d.TargetId))
	}
	if d.Type != "" {
		upd.SetType(loginpolicy.Type(d.Type))
	}
	if d.Method != "" {
		upd.SetMethod(loginpolicy.Method(d.Method))
	}
	if d.Value != "" {
		upd.SetValue(d.Value)
	}
	if d.Reason != "" {
		upd.SetReason(d.Reason)
	}

	if _, err := upd.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("update login policy failed: %v", err)
		return xerr.ServerErrorMsg("update login policy failed")
	}

	return nil
}
