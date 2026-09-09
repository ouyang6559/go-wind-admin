// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package config

import (
	"github.com/zeromicro/go-zero/core/stores/cache"
	"github.com/zeromicro/go-zero/core/stores/redis"
	"github.com/zeromicro/go-zero/core/stores/sqlx"
	"github.com/zeromicro/go-zero/rest"
)

type Config struct {
	rest.RestConf

	// 数据库配置（与 backend 保持一致：PostgreSQL gwa）
	Mysql sqlx.SqlConf
	// Redis 直连配置（与 backend 保持一致）
	Redis redis.RedisConf
	// Redis 缓存集群配置（cache.CacheConf，多节点继续追加即可）
	Cache cache.CacheConf

	// JWT 鉴权配置：.api 中 @server(jwt: Auth) 生成的代码引用
	// serverCtx.Config.Auth.AccessSecret / serverCtx.Config.Auth.AccessExpire
	Auth struct {
		AccessSecret string
		AccessExpire int64
	}
}
