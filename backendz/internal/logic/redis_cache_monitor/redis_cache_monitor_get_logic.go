package redis_cache_monitor

import (
	"context"
	"fmt"
	"strings"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type RedisCacheMonitorGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewRedisCacheMonitorGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *RedisCacheMonitorGetLogic {
	return &RedisCacheMonitorGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *RedisCacheMonitorGetLogic) RedisCacheMonitorGet() (resp *types.RedisCacheMonitorInfo, err error) {
	resp = &types.RedisCacheMonitorInfo{}

	// INFO：尽力读取并解析常用监控段；失败返回空数据而非报错（监控类接口 best-effort）。
	raw, rerr := l.svcCtx.Rds.DoCtx(l.ctx, "INFO")
	if rerr != nil {
		logx.WithContext(l.ctx).Errorf("redis info failed: %v", rerr)
		return resp, nil
	}
	if s, ok := raw.(string); ok {
		resp.Sections = parseRedisInfo(s)
	}

	if dbs, derr := l.svcCtx.Rds.DoCtx(l.ctx, "DBSIZE"); derr == nil {
		resp.DbSize = fmt.Sprintf("%v", dbs)
	}
	return resp, nil
}

// parseRedisInfo 将 redis INFO 文本解析为通用 section/entry 结构。
// 格式：以 "# SectionName" 分段，段内每行 "key:value"，空行分隔。
func parseRedisInfo(info string) []types.InfoSection {
	lines := strings.Split(info, "\n")
	var sections []types.InfoSection
	var cur *types.InfoSection
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		if strings.HasPrefix(line, "#") {
			if cur != nil && len(cur.Entries) > 0 {
				sections = append(sections, *cur)
			}
			cur = &types.InfoSection{Name: strings.TrimSpace(strings.TrimPrefix(line, "#"))}
			continue
		}
		if cur == nil {
			continue
		}
		key, value, found := strings.Cut(line, ":")
		if !found {
			continue
		}
		cur.Entries = append(cur.Entries, types.InfoEntry{Key: strings.TrimSpace(key), Value: strings.TrimSpace(value)})
	}
	if cur != nil && len(cur.Entries) > 0 {
		sections = append(sections, *cur)
	}
	return sections
}