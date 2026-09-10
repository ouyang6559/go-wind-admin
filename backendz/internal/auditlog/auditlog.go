// Package auditlog 提供五类审计（登录/API/操作/权限/数据访问）的 best-effort 写入助手。
// 所有写入均为 best-effort：失败仅 logx.Errorf 打印原始错误（本仓铁律：不吞错），绝不阻断业务。
// 写入统一使用平台上下文（viewer.Default）并 SetTenantID(0)，配合 privacy.DecisionContext(Allow)
// 才能绕过五个审计实体上的 TenantMutationGuardPolicy 租户变更守卫。
package auditlog

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"time"

	"entgo.io/ent/privacy"
	gocrudviewer "github.com/tx7do/go-crud/viewer"
	"github.com/zeromicro/go-zero/core/logx"

	"go-wind-admin/backendz/internal/ent/gen/dataaccessauditlog"
	"go-wind-admin/backendz/internal/ent/gen/loginauditlog"
	"go-wind-admin/backendz/internal/ent/gen/operationauditlog"
	"go-wind-admin/backendz/internal/ent/gen/permissionauditlog"
	"go-wind-admin/backendz/internal/pkg/viewer"
	"go-wind-admin/backendz/internal/svc"
)

// allowCtx 构造带"平台视图 + Allow 决策"的上下文，使租户隐私策略放行审计写入。
func allowCtx(ctx context.Context) context.Context {
	base := gocrudviewer.WithContext(ctx, viewer.Default)
	return privacy.DecisionContext(base, privacy.Allow)
}

// NewRequestID 生成轻量随机请求 ID，用于各审计请求追踪字段（每请求唯一即可）。
func NewRequestID() string {
	b := make([]byte, 8)
	if _, err := rand.Read(b); err == nil {
		return hex.EncodeToString(b)
	}
	return fmt.Sprintf("%d", time.Now().UnixNano())
}

// WriteLogin 写入登录/登出审计（sys_login_audit_logs）。
func WriteLogin(ctx context.Context, s *svc.ServiceContext, a LoginAudit) {
	if a.Method == "" {
		a.Method = loginauditlog.LoginMethodPassword
	}
	if a.RequestID == "" {
		a.RequestID = NewRequestID()
	}

	b := s.Ent.LoginAuditLog.Create().
		SetTenantID(0).
		SetUsername(a.Username).
		SetIPAddress(a.IP).
		SetRequestID(a.RequestID).
		SetActionType(a.Action).
		SetStatus(a.Status).
		SetLoginMethod(a.Method)
	if a.UserID > 0 {
		b.SetUserID(a.UserID)
	}
	if a.SessionID != "" {
		b.SetSessionID(a.SessionID)
	}
	if a.FailureReason != "" {
		b.SetFailureReason(a.FailureReason)
	}
	if _, err := b.Save(allowCtx(ctx)); err != nil {
		logx.WithContext(ctx).Errorf("write login audit (action=%s status=%s) failed: %v", a.Action, a.Status, err)
	}
}

// WriteLoginSuccess 便捷：登录成功，PASSWORD 方式。
func WriteLoginSuccess(ctx context.Context, s *svc.ServiceContext, userID uint32, username, ip, requestID string) {
	WriteLogin(ctx, s, LoginAudit{
		UserID:    userID,
		Username:  username,
		IP:        ip,
		RequestID: requestID,
		Action:    loginauditlog.ActionTypeLogin,
		Status:    loginauditlog.StatusSuccess,
		Method:    loginauditlog.LoginMethodPassword,
	})
}

// WriteLoginFail 便捷：登录失败（PASSWORD 方式）。
func WriteLoginFail(ctx context.Context, s *svc.ServiceContext, username, ip, reason, requestID string) {
	WriteLogin(ctx, s, LoginAudit{
		Username:      username,
		IP:            ip,
		RequestID:     requestID,
		Action:        loginauditlog.ActionTypeLogin,
		Status:        loginauditlog.StatusFailed,
		Method:        loginauditlog.LoginMethodPassword,
		FailureReason: reason,
	})
}

// WriteLogout 便捷：登出。
func WriteLogout(ctx context.Context, s *svc.ServiceContext, userID uint32, username, ip string) {
	WriteLogin(ctx, s, LoginAudit{
		UserID:   userID,
		Username: username,
		IP:       ip,
		Action:   loginauditlog.ActionTypeLogout,
		Status:   loginauditlog.StatusSuccess,
		Method:   loginauditlog.LoginMethodPassword,
	})
}

// WriteAPI 写入 API 访问审计（sys_api_audit_logs）。
func WriteAPI(ctx context.Context, s *svc.ServiceContext, a APIRequest) {
	if a.RequestID == "" {
		a.RequestID = NewRequestID()
	}
	b := s.Ent.ApiAuditLog.Create().
		SetTenantID(0).
		SetIPAddress(a.IP).
		SetHTTPMethod(a.Method).
		SetPath(a.Path).
		SetRequestID(a.RequestID).
		SetLatencyMs(a.LatencyMS).
		SetSuccess(a.Success).
		SetStatusCode(a.StatusCode)
	if a.UserID > 0 {
		b.SetUserID(a.UserID)
	}
	if a.Username != "" {
		b.SetUsername(a.Username)
	}
	if a.RequestURI != "" {
		b.SetRequestURI(a.RequestURI)
	}
	if a.Reason != "" {
		b.SetReason(a.Reason)
	}
	if _, err := b.Save(allowCtx(ctx)); err != nil {
		logx.WithContext(ctx).Errorf("write api audit (method=%s path=%s status=%d) failed: %v", a.Method, a.Path, a.StatusCode, err)
	}
}

// WriteOperation 写入操作审计（sys_operation_audit_logs）。
func WriteOperation(ctx context.Context, s *svc.ServiceContext, a OperationAudit) {
	b := s.Ent.OperationAuditLog.Create().
		SetTenantID(0).
		SetResourceType(a.ResourceType).
		SetResourceID(a.ResourceID).
		SetAction(a.Action).
		SetSuccess(a.Success)
	if a.UserID > 0 {
		b.SetUserID(a.UserID)
	}
	if a.Username != "" {
		b.SetUsername(a.Username)
	}
	if a.BeforeData != "" {
		b.SetBeforeData(a.BeforeData)
	}
	if a.AfterData != "" {
		b.SetAfterData(a.AfterData)
	}
	if a.FailureReason != "" {
		b.SetFailureReason(a.FailureReason)
	}
	if a.IP != "" {
		b.SetIPAddress(a.IP)
	}
	if a.RequestID != "" {
		b.SetRequestID(a.RequestID)
	}
	if _, err := b.Save(allowCtx(ctx)); err != nil {
		logx.WithContext(ctx).Errorf("write operation audit (resource=%s/%s action=%s) failed: %v", a.ResourceType, a.ResourceID, a.Action, err)
	}
}

// WritePermissionChange 写入权限变更审计（sys_permission_audit_logs）。
func WritePermissionChange(ctx context.Context, s *svc.ServiceContext, a PermissionAudit) {
	b := s.Ent.PermissionAuditLog.Create().
		SetTenantID(0).
		SetTargetType(a.TargetType).
		SetTargetID(a.TargetID).
		SetTargetName(a.TargetName).
		SetAction(a.Action)
	if a.OperatorID > 0 {
		b.SetOperatorID(a.OperatorID)
	}
	if a.OperatorName != "" {
		b.SetOperatorName(a.OperatorName)
	}
	if a.OldValue != "" {
		b.SetOldValue(a.OldValue)
	}
	if a.NewValue != "" {
		b.SetNewValue(a.NewValue)
	}
	if a.IP != "" {
		b.SetIPAddress(a.IP)
	}
	if a.RequestID != "" {
		b.SetRequestID(a.RequestID)
	}
	if a.Reason != "" {
		b.SetReason(a.Reason)
	}
	if _, err := b.Save(allowCtx(ctx)); err != nil {
		logx.WithContext(ctx).Errorf("write permission audit (target=%s/%s action=%s) failed: %v", a.TargetType, a.TargetID, a.Action, err)
	}
}

// WriteDataAccess 写入数据访问审计（sys_data_access_audit_logs）。
func WriteDataAccess(ctx context.Context, s *svc.ServiceContext, a DataAccessAudit) {
	b := s.Ent.DataAccessAuditLog.Create().
		SetTenantID(0).
		SetTableName(a.TableName).
		SetDataID(a.DataID).
		SetAccessType(a.AccessType).
		SetSuccess(a.Success)
	if a.UserID > 0 {
		b.SetUserID(a.UserID)
	}
	if a.Username != "" {
		b.SetUsername(a.Username)
	}
	if a.IP != "" {
		b.SetIPAddress(a.IP)
	}
	if a.RequestID != "" {
		b.SetRequestID(a.RequestID)
	}
	if a.SQLDigest != "" {
		b.SetSQLDigest(a.SQLDigest)
	}
	// 显式记录是否脱敏，避免 NULL 语义引起歧义。
	b.SetDataMasked(a.DataMasked)
	if _, err := b.Save(allowCtx(ctx)); err != nil {
		logx.WithContext(ctx).Errorf("write data access audit (table=%s data_id=%s access=%s) failed: %v", a.TableName, a.DataID, a.AccessType, err)
	}
}

// LoginAudit 登录/登出审计入参。
type LoginAudit struct {
	UserID        uint32
	Username      string
	IP            string
	SessionID     string
	RequestID     string
	Action        loginauditlog.ActionType
	Status        loginauditlog.Status
	Method        loginauditlog.LoginMethod
	FailureReason string
}

// APIRequest API 审计入参。
type APIRequest struct {
	UserID     uint32
	Username   string
	IP         string
	Method     string
	Path       string
	RequestURI string
	RequestID  string
	LatencyMS  uint32
	Success    bool
	StatusCode uint32
	Reason     string
}

// OperationAudit 操作审计入参。
type OperationAudit struct {
	UserID        uint32
	Username      string
	ResourceType  string
	ResourceID    string
	Action        operationauditlog.Action
	BeforeData    string
	AfterData     string
	Success       bool
	FailureReason string
	IP            string
	RequestID     string
}

// PermissionAudit 权限变更审计入参。
type PermissionAudit struct {
	OperatorID   uint32
	OperatorName string
	TargetType   string
	TargetID     string
	TargetName   string
	Action       permissionauditlog.Action
	OldValue     string
	NewValue     string
	Reason       string
	IP           string
	RequestID    string
}

// DataAccessAudit 数据访问审计入参。
type DataAccessAudit struct {
	UserID     uint32
	Username   string
	IP         string
	RequestID  string
	TableName  string
	DataID     string
	AccessType dataaccessauditlog.AccessType
	SQLDigest  string
	Success    bool
	DataMasked bool
}