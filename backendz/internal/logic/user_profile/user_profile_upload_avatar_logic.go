// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user_profile

import (
	"context"
	"encoding/base64"
	"strings"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserProfileUploadAvatarLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserProfileUploadAvatarLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserProfileUploadAvatarLogic {
	return &UserProfileUploadAvatarLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserProfileUploadAvatarLogic) UserProfileUploadAvatar(req *types.UploadAvatarRequest) (resp *types.UploadAvatarResponse, err error) {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return nil, xerr.UnauthorizedMsg("unauthorized")
	}

	var avatarURL string
	switch {
	case strings.TrimSpace(req.ImageUrl) != "":
		avatarURL = strings.TrimSpace(req.ImageUrl)
	case req.ImageBase64 != "":
		// 网关未接入 OSS：解码校验后以 data URI 落库，便于本地/单机部署直接展示。
		img, derr := base64.StdEncoding.DecodeString(req.ImageBase64)
		if derr != nil {
			return nil, xerr.BadRequestMsg("invalid avatar base64 data")
		}
		if len(img) == 0 {
			return nil, xerr.BadRequestMsg("empty avatar data")
		}
		avatarURL = "data:image/png;base64," + req.ImageBase64
	default:
		return nil, xerr.BadRequestMsg("invalid avatar source")
	}

	if _, uerr := l.svcCtx.Ent.User.UpdateOneID(c.UserID).
		SetAvatar(avatarURL).
		Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update avatar failed: %v", uerr)
		return nil, xerr.ServerErrorMsg("update avatar failed")
	}
	return &types.UploadAvatarResponse{Url: avatarURL}, nil
}
