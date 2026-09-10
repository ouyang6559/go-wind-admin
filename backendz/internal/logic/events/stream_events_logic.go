// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package events

import (
	"context"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type StreamEventsLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewStreamEventsLogic(ctx context.Context, svcCtx *svc.ServiceContext) *StreamEventsLogic {
	return &StreamEventsLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *StreamEventsLogic) StreamEvents(client chan<- *types.SseMessage) error {
	// todo: add your logic here and delete this line

	return nil
}
