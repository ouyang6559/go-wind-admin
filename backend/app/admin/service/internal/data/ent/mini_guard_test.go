package ent

import (
	"context"
	"testing"

	"github.com/tx7do/go-crud/viewer"
)

func TestMiniCreate(t *testing.T) {
	client := openGuardTestClient(t)
	ctx1 := tenantCtx(context.Background(), 1)
	if _, ok := viewer.FromContext(ctx1); !ok {
		t.Fatal("viewer missing in ctx1")
	}
	client.DictType.Create().SetTypeCode("MINI").SetTypeName("mini").SetIsEnabled(true).ExecX(ctx1)
	t.Log("mini create ok")
}
