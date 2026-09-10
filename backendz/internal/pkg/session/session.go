// Package session 提供基于 Redis 的在线会话注册表。
//
// 会话以「令牌对（access+refresh）」为粒度：登录/刷新轮换签发令牌时写入一条会话记录，
// 随 refresh token 过期（TTL）或被吊销时消失。Redis 键结构：
//
//	gowind:session:<uid>:<jti>   （hash，字段见 Meta）
//
// jti 为令牌对唯一 ID，天然标识一次会话；吊销/存在性校验只需按 (uid, jti) 定位键，
// 无需像 Kratos 那样额外拼接 clientType。
package session

import (
	"context"
	"fmt"
	"sort"
	"strconv"
	"time"

	"github.com/zeromicro/go-zero/core/stores/redis"
)

const keyPrefix = "gowind:session:"

// 缺省会话存活时长，与 refresh token 有效期保持一致（30 天）。
const defaultTTL = 30 * 24 * time.Hour

// Meta 一次在线会话的元数据。
type Meta struct {
	UID        uint32
	TenantID   uint32
	Username   string
	JTI        string    // 令牌对 ID
	ClientType string    // 客户端类型（取 client_id，如 web/mobile）
	IP         string    // 登录 IP
	UserAgent  string    // 浏览器 User-Agent
	DeviceID   string    // 设备 ID
	LoginAt    time.Time // 登录时间（刷新轮换保留首次登录时间）
}

// Manager 会话注册表。
type Manager struct {
	rds *redis.Redis
	ttl time.Duration
}

// NewManager 创建会话注册表，会话默认存活 30 天。
func NewManager(rds *redis.Redis) *Manager {
	return &Manager{rds: rds, ttl: defaultTTL}
}

// Key 返回某用户某会话在 Redis 中的键。
func Key(uid uint32, jti string) string {
	return fmt.Sprintf("%s%d:%s", keyPrefix, uid, jti)
}

// Record 写入一条会话记录（不存在则新建，存在则刷新）。失败返回错误。
func (m *Manager) Record(ctx context.Context, meta Meta) error {
	key := Key(meta.UID, meta.JTI)
	fields := map[string]string{
		"uid":        strconv.FormatUint(uint64(meta.UID), 10),
		"tid":        strconv.FormatUint(uint64(meta.TenantID), 10),
		"username":   meta.Username,
		"jti":        meta.JTI,
		"clientType": meta.ClientType,
		"ip":         meta.IP,
		"userAgent":  meta.UserAgent,
		"deviceId":   meta.DeviceID,
		"loginAt":    strconv.FormatInt(meta.LoginAt.Unix(), 10),
	}
	for f, v := range fields {
		if err := m.rds.HsetCtx(ctx, key, f, v); err != nil {
			return fmt.Errorf("session record hset %s failed: %w", f, err)
		}
	}
	if err := m.rds.ExpireCtx(ctx, key, int(m.ttl.Seconds())); err != nil {
		return fmt.Errorf("session record expire failed: %w", err)
	}
	return nil
}

// List 返回匹配 pattern 的全部会话（按登录时间倒序）。
// pattern 传 "*" 则列出所有人会话，传 "<uid>:*" 则仅列出该用户会话。
func (m *Manager) List(ctx context.Context, pattern string) ([]Meta, error) {
	keys, err := m.rds.Keys(keyPrefix + pattern)
	if err != nil {
		return nil, fmt.Errorf("session keys scan failed: %w", err)
	}
	items := make([]Meta, 0, len(keys))
	for _, k := range keys {
		meta, ok, err := m.GetByKey(ctx, k)
		if err != nil {
			continue
		}
		if ok {
			items = append(items, *meta)
		}
	}
	sortDesc(items)
	return items, nil
}

// Get 按 (uid, jti) 查询会话；不存在返回 (nil,false)。
func (m *Manager) Get(ctx context.Context, uid uint32, jti string) (*Meta, bool, error) {
	return m.GetByKey(ctx, Key(uid, jti))
}

// GetByKey 按完整 Redis 键查询会话。
func (m *Manager) GetByKey(ctx context.Context, key string) (*Meta, bool, error) {
	fields, err := m.rds.HgetallCtx(ctx, key)
	if err != nil {
		return nil, false, fmt.Errorf("session hgetall failed: %w", err)
	}
	if len(fields) == 0 {
		return nil, false, nil
	}
	uid, _ := strconv.ParseUint(fields["uid"], 10, 32)
	tid, _ := strconv.ParseUint(fields["tid"], 10, 32)
	loginAt, _ := strconv.ParseInt(fields["loginAt"], 10, 64)
	return &Meta{
		UID:        uint32(uid),
		TenantID:   uint32(tid),
		Username:   fields["username"],
		JTI:        fields["jti"],
		ClientType: fields["clientType"],
		IP:         fields["ip"],
		UserAgent:  fields["userAgent"],
		DeviceID:   fields["deviceId"],
		LoginAt:    time.Unix(loginAt, 0),
	}, true, nil
}

// Exists 会话是否存在（用于 my-sessions/revoke 的存在性校验）。
func (m *Manager) Exists(ctx context.Context, uid uint32, jti string) (bool, error) {
	ok, err := m.rds.ExistsCtx(ctx, Key(uid, jti))
	return ok, err
}

// Revoke 吊销指定会话（删除访问+刷新令牌对的会话记录）。
func (m *Manager) Revoke(ctx context.Context, uid uint32, jti string) error {
	_, err := m.rds.DelCtx(ctx, Key(uid, jti))
	if err != nil {
		return fmt.Errorf("session revoke failed: %w", err)
	}
	return nil
}

// RevokeByUser 吊销某用户的全部会话（如忘记密码重置后强制全部下线）。
// 返回被吊销的会话数量。
func (m *Manager) RevokeByUser(ctx context.Context, uid uint32) (int, error) {
	keys, err := m.rds.Keys(keyPrefix + fmt.Sprintf("%d:*", uid))
	if err != nil {
		return 0, fmt.Errorf("session user keys scan failed: %w", err)
	}
	for _, k := range keys {
		if _, err := m.rds.DelCtx(ctx, k); err != nil {
			return 0, fmt.Errorf("session user revoke failed: %w", err)
		}
	}
	return len(keys), nil
}

func sortDesc(items []Meta) {
	sort.Slice(items, func(i, j int) bool {
		return items[i].LoginAt.After(items[j].LoginAt)
	})
}
