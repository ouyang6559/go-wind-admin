// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package server_monitor

import (
	"context"
	"os"
	"runtime"
	"time"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type ServerMonitorGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewServerMonitorGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ServerMonitorGetLogic {
	return &ServerMonitorGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// ServerMonitorGet 聚合 Go 运行时 / 数据库连接池 / 主机信息（只读监控视图）。
// 任一子项失败不影响其它子项（数据库兜底为错误信息段）。
func (l *ServerMonitorGetLogic) ServerMonitorGet() (resp *types.ServerMonitorInfo, err error) {
	info := &types.ServerMonitorInfo{
		Go:          l.goRuntimeInfo(),
		Host:        l.hostInfo(),
		CollectedAt: time.Now().UTC().Format(time.RFC3339),
	}
	info.Database = l.databaseInfo()
	return info, nil
}

func (l *ServerMonitorGetLogic) goRuntimeInfo() *types.GoRuntimeInfo {
	var ms runtime.MemStats
	runtime.ReadMemStats(&ms)
	return &types.GoRuntimeInfo{
		Version:       runtime.Version(),
		NumGoroutine:  uint32(runtime.NumGoroutine()),
		MemAllocBytes: ms.Alloc,
		MemSysBytes:   ms.Sys,
		GcCycles:      uint32(ms.NumGC),
		UptimeSeconds: uint64(time.Since(l.svcCtx.StartedAt).Seconds()),
		StartedAt:     l.svcCtx.StartedAt.UTC().Format(time.RFC3339),
	}
}

func (l *ServerMonitorGetLogic) databaseInfo() *types.DatabaseInfo {
	info := &types.DatabaseInfo{Driver: l.svcCtx.DBDriver}

	db := l.svcCtx.DB
	if db == nil {
		info.PingError = "database client is not configured"
		return info
	}

	stats := db.Stats()
	info.MaxOpenConnections = uint32(stats.MaxOpenConnections)
	info.OpenConnections = uint32(stats.OpenConnections)
	info.InUseConnections = uint32(stats.InUse)
	info.IdleConnections = uint32(stats.Idle)

	pingCtx, cancel := context.WithTimeout(l.ctx, 3*time.Second)
	defer cancel()
	if err := db.PingContext(pingCtx); err != nil {
		info.PingError = err.Error()
		logx.WithContext(l.ctx).Errorf("database ping failed: %v", err)
		return info
	}

	info.PingOk = true
	return info
}

func (l *ServerMonitorGetLogic) hostInfo() *types.HostInfo {
	hostname, _ := os.Hostname()
	return &types.HostInfo{
		Os:       runtime.GOOS,
		Arch:     runtime.GOARCH,
		NumCpu:   uint32(runtime.NumCPU()),
		Hostname: hostname,
	}
}
