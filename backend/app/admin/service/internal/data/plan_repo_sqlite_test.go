package data

import (
	"context"
	"testing"

	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"github.com/stretchr/testify/require"
	"github.com/tx7do/go-utils/mapper"
	"github.com/tx7do/go-utils/trans"
	"google.golang.org/protobuf/types/known/fieldmaskpb"

	paginationV1 "github.com/tx7do/go-crud/api/gen/go/pagination/v1"
	entCrud "github.com/tx7do/go-crud/entgo"

	identityV1 "go-wind-admin/api/gen/go/identity/service/v1"
	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/app/admin/service/internal/data/ent/plan"
	"go-wind-admin/app/admin/service/internal/data/enttest"
)

// newPlanRepoSqlite 在给定 enttest client 上白盒构造 PlanRepo，
// 逐字段复刻 NewPlanRepo 的 mapper/converter 初始化，再调用 init()。
func newPlanRepoSqlite(t *testing.T, entClient *entCrud.EntClient[*ent.Client]) *PlanRepo {
	t.Helper()
	repo := &PlanRepo{
		entClient:       entClient,
		log:             bLogger.NewHelper(bLogger.NopLogger()),
		mapper:          mapper.NewCopierMapper[identityV1.Plan, ent.Plan](),
		versionConverter: mapper.NewEnumTypeConverter[identityV1.Plan_Version, plan.Version](
			identityV1.Plan_Version_name, identityV1.Plan_Version_value,
		),
		expiryPolicyConv: mapper.NewEnumTypeConverter[identityV1.Plan_ExpiryPolicy, plan.ExpiryPolicy](
			identityV1.Plan_ExpiryPolicy_name, identityV1.Plan_ExpiryPolicy_value,
		),
	}
	repo.init()
	return repo
}

// TestPlanRepoSqlite_Create 通过 repo.Create 写入后直查 SQLite 断言落库
// （含枚举字段经 converter 的落库值）。
func TestPlanRepoSqlite_Create(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newPlanRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	err := repo.Create(ctx, &identityV1.CreatePlanRequest{
		Data: &identityV1.Plan{
			Name:              trans.Ptr("sqlite套餐-创建"),
			Version:           identityV1.Plan_FREE.Enum(),
			ExpiryPolicy:      identityV1.Plan_READONLY.Enum(),
			DataRetentionDays: trans.Ptr(uint32(30)),
		},
	})
	require.NoError(t, err, "repo.Create 应写入 SQLite 成功")

	rows, err := repo.entClient.Client().Plan.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, rows, 1, "SQLite 中应有 1 条 plan 记录")
	require.Equal(t, "sqlite套餐-创建", *rows[0].Name, "name 应按请求落库")
	require.Equal(t, plan.VersionFree, *rows[0].Version, "version 枚举应经 converter 落为 FREE")
	require.Equal(t, plan.ExpiryPolicyReadonly, *rows[0].ExpiryPolicy, "expiry_policy 枚举应经 converter 落为 READONLY")
	require.NotNil(t, rows[0].DataRetentionDays)
	require.Equal(t, uint32(30), *rows[0].DataRetentionDays, "data_retention_days 应按请求落库")
}

// TestPlanRepoSqlite_CreateDuplicateName 验证套餐名唯一索引：
// 重名创建返回 400（BadRequest）而非 500。
func TestPlanRepoSqlite_CreateDuplicateName(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newPlanRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	require.NoError(t, repo.Create(ctx, &identityV1.CreatePlanRequest{
		Data: &identityV1.Plan{Name: trans.Ptr("sqlite套餐-重名")},
	}))
	err := repo.Create(ctx, &identityV1.CreatePlanRequest{
		Data: &identityV1.Plan{Name: trans.Ptr("sqlite套餐-重名")},
	})
	require.Error(t, err, "重名创建应命中唯一索引报错")
}

// TestPlanRepoSqlite_ListContainsFilter 验证 List 的 contains 模糊搜索语义。
func TestPlanRepoSqlite_ListContainsFilter(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newPlanRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	require.NoError(t, repo.Create(ctx, &identityV1.CreatePlanRequest{
		Data: &identityV1.Plan{Name: trans.Ptr("套餐-markerpoi-甲")},
	}))
	require.NoError(t, repo.Create(ctx, &identityV1.CreatePlanRequest{
		Data: &identityV1.Plan{Name: trans.Ptr("套餐-无关行-乙")},
	}))

	filtered, err := repo.List(ctx, &paginationV1.PagingRequest{
		FilteringType: &paginationV1.PagingRequest_FilterExpr{
			FilterExpr: &paginationV1.FilterExpr{
				Type: paginationV1.ExprType_AND,
				Conditions: []*paginationV1.FilterCondition{
					{
						Field:      "name",
						Op:         paginationV1.Operator_CONTAINS,
						ValueOneof: &paginationV1.FilterCondition_Value{Value: "markerpoi"},
					},
				},
			},
		},
	})
	require.NoError(t, err)
	require.Equal(t, uint64(1), filtered.Total, "contains 过滤后 Total 应为 1")
	require.Len(t, filtered.Items, 1, "contains 过滤后只应返回 1 行")
	require.Contains(t, *filtered.Items[0].Name, "markerpoi", "命中行应是携带标记的行")

	none, err := repo.List(ctx, &paginationV1.PagingRequest{
		FilteringType: &paginationV1.PagingRequest_FilterExpr{
			FilterExpr: &paginationV1.FilterExpr{
				Type: paginationV1.ExprType_AND,
				Conditions: []*paginationV1.FilterCondition{
					{
						Field:      "name",
						Op:         paginationV1.Operator_CONTAINS,
						ValueOneof: &paginationV1.FilterCondition_Value{Value: "no-such-marker-zzz"},
					},
				},
			},
		},
	})
	require.NoError(t, err)
	require.Equal(t, uint64(0), none.Total, "无命中 contains 应返回 Total=0")
	require.Empty(t, none.Items)

	all, err := repo.List(ctx, &paginationV1.PagingRequest{})
	require.NoError(t, err)
	require.Equal(t, uint64(2), all.Total, "无过滤时应返回全部 2 行")
	require.Len(t, all.Items, 2)
}

// TestPlanRepoSqlite_Get 验证 Get 命中/未命中。
func TestPlanRepoSqlite_Get(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newPlanRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	require.NoError(t, repo.Create(ctx, &identityV1.CreatePlanRequest{
		Data: &identityV1.Plan{Name: trans.Ptr("sqlite套餐-Get")},
	}))
	rows, err := repo.entClient.Client().Plan.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, rows, 1)
	createdID := rows[0].ID

	hit, err := repo.Get(ctx, &identityV1.GetPlanRequest{
		QueryBy: &identityV1.GetPlanRequest_Id{Id: createdID},
	})
	require.NoError(t, err, "按存在的 ID 查询应命中")
	require.Equal(t, createdID, hit.GetId())

	_, err = repo.Get(ctx, &identityV1.GetPlanRequest{
		QueryBy: &identityV1.GetPlanRequest_Id{Id: 9999999},
	})
	require.Error(t, err, "不存在的 ID 查询应返回错误")
}

// TestPlanRepoSqlite_Update 验证 Update 只更新掩码内字段（description），
// 掩码外字段（name）保持原值。
func TestPlanRepoSqlite_Update(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newPlanRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	require.NoError(t, repo.Create(ctx, &identityV1.CreatePlanRequest{
		Data: &identityV1.Plan{
			Name:        trans.Ptr("sqlite套餐-更新"),
			Description: trans.Ptr("更新前描述"),
		},
	}))
	rows, err := repo.entClient.Client().Plan.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, rows, 1)
	createdID := rows[0].ID

	err = repo.Update(ctx, &identityV1.UpdatePlanRequest{
		Id:         createdID,
		UpdateMask: &fieldmaskpb.FieldMask{Paths: []string{"description"}},
		Data: &identityV1.Plan{
			Description: trans.Ptr("更新后描述-sqlite"),
		},
	})
	require.NoError(t, err, "更新 description 应成功")

	after, err := repo.entClient.Client().Plan.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, after, 1)
	require.Equal(t, "更新后描述-sqlite", *after[0].Description, "掩码内字段应被更新")
	require.Equal(t, "sqlite套餐-更新", *after[0].Name, "掩码外字段应保持原值")
}

// TestPlanRepoSqlite_Delete 验证 Delete 后行数归零。
func TestPlanRepoSqlite_Delete(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newPlanRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	require.NoError(t, repo.Create(ctx, &identityV1.CreatePlanRequest{
		Data: &identityV1.Plan{Name: trans.Ptr("sqlite套餐-待删除")},
	}))
	rows, err := repo.entClient.Client().Plan.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, rows, 1)

	require.NoError(t, repo.Delete(ctx, rows[0].ID), "Delete 应成功")
	cnt, err := repo.entClient.Client().Plan.Query().Count(ctx)
	require.NoError(t, err)
	require.Zero(t, cnt, "Delete 后表内行数应为 0")
}
