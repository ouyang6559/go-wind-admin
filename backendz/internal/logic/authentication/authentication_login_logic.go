package authentication

import (
	"context"
	"net"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/auditlog"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/credential"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/pkg/session"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

const (
	headerCaptchaID    = "X-Captcha-Id"
	headerCaptchaValue = "X-Captcha-Value"
)

type AuthenticationLoginLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAuthenticationLoginLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AuthenticationLoginLogic {
	return &AuthenticationLoginLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *AuthenticationLoginLogic) AuthenticationLogin(req *types.LoginRequest) (resp *types.LoginResponse, err error) {
	requestID := auditlog.NewRequestID()
	ip := l.clientIP()

	// 登录审计：无论成功/失败统一在返回前写出一次（best-effort，失败仅记日志不阻断登录）。
	var username string
	var auditUserID uint32
	defer func() {
		if err != nil {
			auditlog.WriteLoginFail(l.ctx, l.svcCtx, username, ip, err.Error(), requestID)
		} else if resp != nil {
			auditlog.WriteLoginSuccess(l.ctx, l.svcCtx, auditUserID, username, ip, requestID)
		}
	}()

	grantType := req.GrantType
	if grantType == "" {
		grantType = "password"
	}
	if grantType != "password" {
		return nil, xerr.InvalidGrantTypeMsg()
	}

	// 验证码：通过请求头传递（X-Captcha-Id / X-Captcha-Value）
	if !l.verifyLoginCaptcha() {
		return nil, xerr.InvalidCaptchaMsg()
	}

	username = strings.NewReplacer("\r", "", "\n", "").Replace(strings.TrimSpace(req.Username))
	if username == "" || req.Password == "" {
		return nil, xerr.BadRequestMsg("username and password required")
	}

	// 租户解析：tenant_code 留空视为平台（0）
	var tenantID uint32
	if code := strings.TrimSpace(req.TenantCode); code != "" {
		t, terr := l.svcCtx.Ent.Tenant.Query().Where(tenant.CodeEQ(code)).First(l.ctx)
		if terr != nil || t.Status == nil || *t.Status != tenant.StatusOn {
			return nil, xerr.BadRequestMsg("invalid tenant")
		}
		tenantID = t.ID
	}

	// 查找用户名凭证
	cred, cerr := l.svcCtx.Ent.UserCredential.Query().
		Where(
			usercredential.TenantIDEQ(tenantID),
			usercredential.IdentifierEQ(username),
			usercredential.IdentityTypeEQ(usercredential.IdentityTypeUsername),
			usercredential.StatusEQ(usercredential.StatusEnabled),
		).
		First(l.ctx)
	if cerr != nil || cred.Credential == nil {
		return nil, xerr.InvalidPasswordMsg()
	}

	// 前端/旧后端约定密码以 AES-128-CBC(base64) 加密传输，先还原明文再 bcrypt 校验；
	// 同时兼容内部接口测试直接提交明文（解密失败时回退用原值）。
	plainPassword := credential.ResolvePlainPassword(req.Password)
	if !password.Verify(*cred.Credential, plainPassword) {
		// 兜底：当解密结果未命中、但原值反而命中时（密文被误判等边界），仍放行
		if req.Password == plainPassword || !password.Verify(*cred.Credential, req.Password) {
			return nil, xerr.InvalidPasswordMsg()
		}
	}

	if cred.UserID == nil {
		return nil, xerr.InvalidPasswordMsg()
	}

	u, uerr := l.svcCtx.Ent.User.Get(l.ctx, *cred.UserID)
	if uerr != nil {
		return nil, xerr.InvalidPasswordMsg()
	}
	if u.Status == nil || *u.Status != user.StatusNormal {
		return nil, xerr.ForbiddenMsg("user is disabled")
	}
	if u.TenantID == nil || *u.TenantID != tenantID {
		return nil, xerr.BadRequestMsg("invalid tenant")
	}

	usernameVal := ""
	if u.Username != nil {
		usernameVal = *u.Username
	}

	accessToken, refreshToken, jti, terr := l.svcCtx.Token.CreateTokenPair(u.ID, *u.TenantID, usernameVal, req.ClientId, req.DeviceId)
	if terr != nil {
		return nil, xerr.ServerErrorMsg("create token pair failed")
	}
	clientType := req.ClientType
	if strings.TrimSpace(clientType) == "" {
		clientType = req.ClientId
	}

	// 写入在线会话注册表（best-effort：Redis 写失败不阻断登录，仅记录日志）
	if serr := l.svcCtx.Session.Record(l.ctx, session.Meta{
		UID:        u.ID,
		TenantID:   *u.TenantID,
		Username:   usernameVal,
		JTI:        jti,
		ClientType: clientType,
		IP:         l.clientIP(),
		UserAgent:  l.userAgent(),
		DeviceID:   req.DeviceId,
		LoginAt:    time.Now(),
	}); serr != nil {
		logx.WithContext(l.ctx).Errorf("record session for user [%d] failed: %v", u.ID, serr)
	}

	// 更新最近登录信息（失败不阻断）
	clientIP := l.clientIP()
	_, _ = l.svcCtx.Ent.User.UpdateOneID(u.ID).
		SetLastLoginAt(time.Now()).
		SetLastLoginIP(clientIP).
		Save(l.ctx)

	// 供登录成功审计使用
	auditUserID = u.ID
	username = usernameVal

	return &types.LoginResponse{
		TokenType:        "Bearer",
		AccessToken:      accessToken,
		ExpiresIn:        l.svcCtx.Token.AccessExpiresIn(),
		RefreshToken:     refreshToken,
		RefreshExpiresIn: l.svcCtx.Token.RefreshExpiresIn(),
	}, nil
}

func (l *AuthenticationLoginLogic) verifyLoginCaptcha() bool {
	r, ok := middleware.RequestFromContext(l.ctx)
	if !ok {
		return false
	}
	id := strings.TrimSpace(r.Header.Get(headerCaptchaID))
	val := strings.TrimSpace(r.Header.Get(headerCaptchaValue))
	if id == "" || val == "" {
		return false
	}
	return l.svcCtx.CaptchaVerify(id, val)
}

func (l *AuthenticationLoginLogic) clientIP() string {
	r, ok := middleware.RequestFromContext(l.ctx)
	if !ok {
		return ""
	}
	// RemoteAddr 形如 "192.168.65.1:50516"，去掉端口仅保留 IP（与 backend/kratos 返回一致）
	if host, _, err := net.SplitHostPort(r.RemoteAddr); err == nil {
		return host
	}
	return r.RemoteAddr
}

func (l *AuthenticationLoginLogic) userAgent() string {
	if r, ok := middleware.RequestFromContext(l.ctx); ok {
		return r.UserAgent()
	}
	return ""
}
