package service

import (
	"context"
	"crypto/rand"
	"crypto/sha256"
	"crypto/subtle"
	"encoding/hex"
	"fmt"
	"time"

	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"google.golang.org/protobuf/types/known/emptypb"
	"google.golang.org/protobuf/types/known/timestamppb"

	adminV1 "go-wind-admin/api/gen/go/admin/service/v1"
	accesskeyV1 "go-wind-admin/api/gen/go/access_key/service/v1"
	authenticationV1 "go-wind-admin/api/gen/go/authentication/service/v1"
	paginationV1 "github.com/tx7do/go-crud/api/gen/go/pagination/v1"
	"go-wind-admin/app/admin/service/internal/data/ent/accesskey"
	"go-wind-admin/app/admin/service/internal/data"
	"github.com/tx7do/kratos-bootstrap/bootstrap"
	"go-wind-admin/pkg/middleware/auth"
	"github.com/tx7do/go-utils/trans"
)

// AK 前缀与长度：AK 公开可见（16 字节 hex），SK 高熵（32 字节 hex）只在创建响应中出现一次。
const (
	accessKeyPrefix = "ak-"
	secretPrefix    = "sk-"
	accessKeyBytes  = 8
	secretBytes     = 32
)

type AccessKeyService struct {
	adminV1.AccessKeyServiceHTTPServer
	log           *bLogger.Helper
	repo          *data.AccessKeyRepo
	authenticator *data.Authenticator
}

func NewAccessKeyService(
	ctx *bootstrap.Context,
	repo *data.AccessKeyRepo,
	authenticator *data.Authenticator,
) *AccessKeyService {
	return &AccessKeyService{
		log:           ctx.NewLoggerHelper("access-key/service/admin-service"),
		repo:          repo,
		authenticator: authenticator,
	}
}

func (s *AccessKeyService) List(ctx context.Context, req *paginationV1.PagingRequest) (*accesskeyV1.ListAccessKeyResponse, error) {
	return s.repo.List(ctx, req)
}

func (s *AccessKeyService) Get(ctx context.Context, req *accesskeyV1.GetAccessKeyRequest) (*accesskeyV1.AccessKey, error) {
	return s.repo.Get(ctx, req)
}

// Create 生成 AK/SK 并落库（SK 只存 SHA-256 摘要），明文 secret 仅本次响应返回。
func (s *AccessKeyService) Create(ctx context.Context, req *accesskeyV1.CreateAccessKeyRequest) (*accesskeyV1.CreateAccessKeyResponse, error) {
	if req == nil || req.Data == nil {
		return nil, adminV1.ErrorBadRequest("invalid parameter")
	}

	operator, err := auth.FromContext(ctx)
	if err != nil {
		return nil, err
	}

	// 凭证归属创建者的租户：机器令牌将继承该租户的隔离与 Api 表闸门
	tenantId := operator.GetTenantId()
	req.Data.TenantId = trans.Ptr(tenantId)

	accessKey, err := randomToken(accessKeyPrefix, accessKeyBytes)
	if err != nil {
		s.log.Errorf(ctx, "generate access key failed: %s", err.Error())
		return nil, adminV1.ErrorInternalServerError("generate access key failed")
	}
	secret, err := randomToken(secretPrefix, secretBytes)
	if err != nil {
		s.log.Errorf(ctx, "generate secret failed: %s", err.Error())
		return nil, adminV1.ErrorInternalServerError("generate secret failed")
	}
	secretHash := hashSecret(secret)

	entity, err := s.repo.Create(ctx, req, req.Data, accessKey, secretHash)
	if err != nil {
		return nil, err
	}

	return &accesskeyV1.CreateAccessKeyResponse{
		Data: &accesskeyV1.AccessKey{
			Id:        trans.Ptr(uint32(entity.ID)),
			Name:      entity.Name,
			AccessKey: entity.AccessKey,
			Status:    s.statusProto(entity.Status),
			ExpiresAt: timestampOf(entity.ExpiresAt),
			TenantId:  entity.TenantID,
			CreatedAt: timestampOf(entity.CreatedAt),
		},
		Secret: secret,
	}, nil
}

// Update 仅允许改名称/状态/过期时间。AK 与 secret 摘要不可变：
// 黑列出 mask 中的非法路径，防止 FieldMask 管线把它们误写。
func (s *AccessKeyService) Update(ctx context.Context, req *accesskeyV1.UpdateAccessKeyRequest) (*emptypb.Empty, error) {
	if req == nil || req.Data == nil {
		return nil, adminV1.ErrorBadRequest("invalid parameter")
	}
	if req.GetId() == 0 {
		return nil, adminV1.ErrorBadRequest("id is required")
	}

	operator, err := auth.FromContext(ctx)
	if err != nil {
		return nil, err
	}
	req.Data.UpdatedBy = trans.Ptr(operator.UserId)
	if req.UpdateMask != nil {
		req.UpdateMask.Paths = append(req.UpdateMask.Paths, "updated_by")
		req.UpdateMask.Paths = append(req.UpdateMask.Paths, "access_key", "secret_hash", "tenant_id")
	}
	if err := s.repo.Update(ctx, req); err != nil {
		return nil, err
	}
	return &emptypb.Empty{}, nil
}

func (s *AccessKeyService) Delete(ctx context.Context, req *accesskeyV1.DeleteAccessKeyRequest) (*emptypb.Empty, error) {
	if err := s.repo.Delete(ctx, req); err != nil {
		return nil, err
	}
	return &emptypb.Empty{}, nil
}

// IssueToken 令牌交换（免鉴权）：校验 AK/SK 后签发租户作用域的机器令牌。
// 令牌 userId=0（机器身份）、tenant=凭证归属租户、无角色——下游的租户隔离、
// Api 表闸门与审计照常生效；authz 启用后机器令牌默认无任何角色权限。
func (s *AccessKeyService) IssueToken(ctx context.Context, req *accesskeyV1.IssueTokenRequest) (*accesskeyV1.IssueTokenResponse, error) {
	if req == nil || req.GetAccessKey() == "" || req.GetSecret() == "" {
		return nil, adminV1.ErrorBadRequest("access key and secret are required")
	}

	entity, err := s.repo.GetByAccessKeyBySystem(ctx, req.GetAccessKey())
	if err != nil {
		// 不区分"不存在"与"密钥错误"，避免泄露 AK 是否有效
		return nil, adminV1.ErrorBadRequest("invalid access key or secret")
	}

	if entity.Status == nil || *entity.Status != accesskey.StatusOn {
		return nil, adminV1.ErrorBadRequest("access key is disabled")
	}
	if entity.ExpiresAt != nil && time.Now().After(*entity.ExpiresAt) {
		return nil, adminV1.ErrorBadRequest("access key is expired")
	}

	sum := sha256.Sum256([]byte(req.GetSecret()))
	given := hex.EncodeToString(sum[:])
	if entity.SecretHash == nil ||
		subtle.ConstantTimeCompare([]byte(given), []byte(*entity.SecretHash)) != 1 {
		return nil, adminV1.ErrorBadRequest("invalid access key or secret")
	}

	payload := &authenticationV1.UserTokenPayload{
		UserId:   0, // 机器身份：无对应用户行
		TenantId: entity.TenantID,
		Username: trans.Ptr("ak:" + req.GetAccessKey()),
		Roles:    []string{"machine"}, // authz 中间件要求非空 subject；noop 引擎全放行，casbin 下默认无策略=fail-closed
	}
	accessToken, expires, err := s.authenticator.CreateMachineToken(ctx, payload)
	if err != nil {
		s.log.Errorf(ctx, "issue machine token failed: %s", err.Error())
		return nil, adminV1.ErrorInternalServerError("issue token failed")
	}

	// 尽力而为刷新最近使用时间（不阻塞响应）
	s.repo.TouchLastUsedBySystem(context.WithoutCancel(ctx), entity.ID)

	return &accesskeyV1.IssueTokenResponse{
		AccessToken: accessToken,
		ExpiresIn:   uint32(expires.Seconds()),
		TokenType:   "bearer",
	}, nil
}

func (s *AccessKeyService) Count(ctx context.Context, req *paginationV1.PagingRequest) (*accesskeyV1.CountAccessKeyResponse, error) {
	return s.repo.Count(ctx, req)
}

func randomToken(prefix string, nBytes int) (string, error) {
	buf := make([]byte, nBytes)
	if _, err := rand.Read(buf); err != nil {
		return "", fmt.Errorf("read random bytes: %w", err)
	}
	return prefix + hex.EncodeToString(buf), nil
}

func hashSecret(secret string) string {
	sum := sha256.Sum256([]byte(secret))
	return hex.EncodeToString(sum[:])
}

func (s *AccessKeyService) statusProto(st *accesskey.Status) *accesskeyV1.AccessKey_Status {
	if st == nil {
		return nil
	}
	if *st == accesskey.StatusOff {
		v := accesskeyV1.AccessKey_OFF
		return &v
	}
	v := accesskeyV1.AccessKey_ON
	return &v
}

func timestampOf(t *time.Time) *timestamppb.Timestamp {
	if t == nil || t.IsZero() {
		return nil
	}
	return timestamppb.New(*t)
}
