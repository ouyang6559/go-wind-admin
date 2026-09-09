// Package ssehub 提供按用户订阅/发布的 SSE 事件 Hub。
//
// 与 backend（kratos 的 sse transport）保持一致：以 userId 作为 streamID，
// 同一用户的所有在线连接订阅同一条流，发送站内信时逐条推送给该用户的所有订阅。
package ssehub

import (
	"sync"
)

// Hub 管理按用户组织的 SSE 订阅通道，并支持非阻塞广播。
// Publish 不阻塞发送方：某个慢客户端缓冲塞满时直接跳过该连接，避免拖垮广播 fan-out。
type Hub struct {
	mu    sync.RWMutex
	users map[uint32]map[chan []byte]struct{}
}

// New 创建空 Hub。
func New() *Hub {
	return &Hub{
		users: make(map[uint32]map[chan []byte]struct{}),
	}
}

// Subscribe 注册给定用户的在线订阅，返回其专属事件通道。
// 该通道为有缓冲（16），调用方需在连接结束时调用 Unsubscribe 释放。
func (h *Hub) Subscribe(userID uint32) chan []byte {
	ch := make(chan []byte, 16)

	h.mu.Lock()
	defer h.mu.Unlock()
	if h.users[userID] == nil {
		h.users[userID] = make(map[chan []byte]struct{})
	}
	h.users[userID][ch] = struct{}{}

	return ch
}

// Unsubscribe 注销订阅并关闭通道，调用方不应再向 ch 读写。
func (h *Hub) Unsubscribe(userID uint32, ch chan []byte) {
	h.mu.Lock()
	defer h.mu.Unlock()

	if set := h.users[userID]; len(set) > 0 {
		if _, ok := set[ch]; ok {
			delete(set, ch)
			close(ch)
		}
		if len(set) == 0 {
			delete(h.users, userID)
		}
	}
}

// Publish 向目标用户的全部在线订阅推送 payload（尽力而为，非阻塞）。
// 返回实际投递到的订阅数；用户无在线 SSE 连接时返回 0。
func (h *Hub) Publish(userID uint32, payload []byte) int {
	h.mu.RLock()
	set := h.users[userID]
	targets := make([]chan []byte, 0, len(set))
	for ch := range set {
		targets = append(targets, ch)
	}
	h.mu.RUnlock()

	delivered := 0
	for _, ch := range targets {
		select {
		case ch <- payload:
			delivered++
		default:
			// 慢客户端缓冲已满，跳过（不阻塞发送方）
		}
	}
	return delivered
}
