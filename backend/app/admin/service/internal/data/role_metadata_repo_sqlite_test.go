package data

import (
	"context"
	"testing"

	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"github.com/stretchr/testify/require"
	"github.com/tx7do/go-utils/mapper"
	"github.com/tx7do/go-utils/trans"

	permissionV1 "go-wind-admin/api/gen/go/permission/service/v1"
	"go-wind-admin/app/admin/service/internal/data/ent"
	entRoleMetadata "go-wind-admin/app/admin/service/internal/data/ent/rolemetadata"
	"go-wind-admin/app/admin/service/internal/data/enttest"
)

// newRoleMetadataRepoSqlite 用 enttest helper 构造一个可直接做 CRUD 的 RoleMetadataRepo。
// 白盒构造逐字段复刻 NewRoleMetadataRepo 的 mapper/converter 初始化，
// 仅将 log 换为 NopLogger、entClient 换为 SQLite 内存库测试 client。
func newRoleMetadataRepoSqlite(t *testing.T) *RoleMetadataRepo {
	t.Helper()
	entClient := enttest.NewEntClientForTest(t)
	repo := &RoleMetadataRepo{
		log:       bLogger.NewHelper(bLogger.NopLogger()),
		entClient: entClient,
		mapper:    mapper.NewCopierMapper[permissionV1.RoleMetadata, ent.RoleMetadata](),
		syncPolicyConverter: mapper.NewEnumTypeConverter[permissionV1.RoleMetadata_SyncPolicy, entRoleMetadata.SyncPolicy](
			permissionV1.RoleMetadata_SyncPolicy_name,
			permissionV1.RoleMetadata_SyncPolicy_value,
		),
		scopeConverter: mapper.NewEnumTypeConverter[permissionV1.RoleMetadata_Scope, entRoleMetadata.Scope](
			permissionV1.RoleMetadata_Scope_name,
			permissionV1.RoleMetadata_Scope_value,
		),
	}

	repo.init()

	return repo
}

// TestRoleMetadataRepoSqlite_CreateGetExist 覆盖 RoleMetadataRepo 的写入与读取：
// Create（落库）→ ent client 直查确认 → Get（命中）→ IsExistByRoleID/IsTemplateRole。
func TestRoleMetadataRepoSqlite_CreateGetExist(t *testing.T) {
	repo := newRoleMetadataRepoSqlite(t)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	const roleID = uint32(16001)

	tx, err := repo.entClient.Client().Tx(ctx)
	require.NoError(t, err)
	// 无论断言成败都释放事务连接：Commit 成功后 Rollback 返回 ErrTxDone 被忽略；
	// 断言失败（Goexit）时回滚，避免残留写锁阻塞后续测试。
	defer func() { _ = tx.Rollback() }()
	require.NoError(t, repo.Create(ctx, tx, &permissionV1.RoleMetadata{
		RoleId:     trans.Ptr(roleID),
		IsTemplate: trans.Ptr(false),
		SyncPolicy: permissionV1.RoleMetadata_AUTO.Enum(),
		Scope:      permissionV1.RoleMetadata_TENANT.Enum(),
	}), "写入角色元数据应成功")
	require.NoError(t, tx.Commit())

	// ent client 直查：记录确实落库，枚举经转换器映射
	rows, err := repo.entClient.Client().RoleMetadata.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, rows, 1, "sys_role_metadata 应有 1 条记录")
	require.Equal(t, roleID, *rows[0].RoleID, "role_id 应按载荷落库")
	require.NotNil(t, rows[0].SyncPolicy, "sync_policy 应经转换器落库")
	require.Equal(t, entRoleMetadata.SyncPolicyAuto, *rows[0].SyncPolicy, "proto AUTO 应映射为 ent SyncPolicyAuto")
	require.NotNil(t, rows[0].Scope, "scope 应经转换器落库")
	require.Equal(t, entRoleMetadata.ScopeTenant, *rows[0].Scope, "proto TENANT 应映射为 ent ScopeTenant")

	// Get 命中：按 roleID 查询单条
	got, err := repo.Get(ctx, roleID)
	require.NoError(t, err, "按 roleID 查询已存在元数据应命中")
	require.Equal(t, roleID, got.GetRoleId(), "命中记录的 role_id 应与写入一致")
	require.False(t, got.GetIsTemplate(), "命中记录应为非模板")

	// 存在性与模板判定
	exist, err := repo.IsExistByRoleID(ctx, roleID)
	require.NoError(t, err)
	require.True(t, exist, "已写入的 roleID 应判定为存在")
	isTpl, err := repo.IsTemplateRole(ctx, roleID)
	require.NoError(t, err)
	require.False(t, isTpl, "非模板记录应判定为非模板")

	// 未命中：Get 不存在的 roleID 应报错
	_, err = repo.Get(ctx, 99999)
	require.Error(t, err, "查询不存在的 roleID 应返回错误")
	notExist, err := repo.IsExistByRoleID(ctx, 99999)
	require.NoError(t, err)
	require.False(t, notExist, "不存在的 roleID 应判定为不存在")
	_, err = repo.IsTemplateRole(ctx, 99999)
	require.Error(t, err, "对不存在的 roleID 判定模板应返回错误")
}

// TestRoleMetadataRepoSqlite_TemplateVersionUpgrade 验证 UpgradeTemplateVersion
// 仅对模板记录生效并递增版本号；非模板记录调用后无变化。
func TestRoleMetadataRepoSqlite_TemplateVersionUpgrade(t *testing.T) {
	repo := newRoleMetadataRepoSqlite(t)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	const (
		templateRoleID = uint32(16002)
		plainRoleID    = uint32(16003)
	)

	// 模板记录 + 普通记录各一条
	tx, err := repo.entClient.Client().Tx(ctx)
	require.NoError(t, err)
	defer func() { _ = tx.Rollback() }()
	require.NoError(t, repo.Create(ctx, tx, &permissionV1.RoleMetadata{
		RoleId:          trans.Ptr(templateRoleID),
		IsTemplate:      trans.Ptr(true),
		TemplateVersion: trans.Ptr(int32(3)),
		SyncPolicy:      permissionV1.RoleMetadata_AUTO.Enum(),
		Scope:           permissionV1.RoleMetadata_PLATFORM.Enum(),
	}))
	require.NoError(t, repo.Create(ctx, tx, &permissionV1.RoleMetadata{
		RoleId:          trans.Ptr(plainRoleID),
		IsTemplate:      trans.Ptr(false),
		TemplateVersion: trans.Ptr(int32(7)),
		SyncPolicy:      permissionV1.RoleMetadata_AUTO.Enum(),
		Scope:           permissionV1.RoleMetadata_TENANT.Enum(),
	}))
	require.NoError(t, tx.Commit())

	// 模板记录：版本号 +1（3 → 4）
	tx2, err := repo.entClient.Client().Tx(ctx)
	require.NoError(t, err)
	defer func() { _ = tx2.Rollback() }()
	require.NoError(t, repo.UpgradeTemplateVersion(ctx, tx2, templateRoleID))
	require.NoError(t, tx2.Commit())

	tplRow, err := repo.entClient.Client().RoleMetadata.Query().
		Where(entRoleMetadata.RoleIDEQ(templateRoleID)).
		Only(ctx)
	require.NoError(t, err)
	require.Equal(t, int32(4), *tplRow.TemplateVersion, "模板记录版本号应递增为 4")

	// 非模板记录：调用不报错但版本号保持不变
	tx3, err := repo.entClient.Client().Tx(ctx)
	require.NoError(t, err)
	defer func() { _ = tx3.Rollback() }()
	require.NoError(t, repo.UpgradeTemplateVersion(ctx, tx3, plainRoleID))
	require.NoError(t, tx3.Commit())

	plainRow, err := repo.entClient.Client().RoleMetadata.Query().
		Where(entRoleMetadata.RoleIDEQ(plainRoleID)).
		Only(ctx)
	require.NoError(t, err)
	require.Equal(t, int32(7), *plainRow.TemplateVersion, "非模板记录版本号应保持 7 不变")
}

// TestRoleMetadataRepoSqlite_Upsert 验证 Upsert 在 SQLite 下的实际行为：
// 其冲突目标仅声明 role_id，而表上唯一的唯一索引是复合索引 (tenant_id, role_id)，
// SQLite 要求 ON CONFLICT 目标必须精确匹配某个唯一索引，因此该语句被整体拒绝、
// 新行也不会插入（记录该现状：Upsert 的冲突目标与 schema 唯一索引不一致）。
func TestRoleMetadataRepoSqlite_Upsert(t *testing.T) {
	repo := newRoleMetadataRepoSqlite(t)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	const roleID = uint32(16004)

	// Upsert 被数据库拒绝：冲突目标不匹配任何唯一索引
	require.Error(t, repo.Upsert(ctx, &permissionV1.RoleMetadata{
		RoleId:          trans.Ptr(roleID),
		IsTemplate:      trans.Ptr(false),
		TemplateVersion: trans.Ptr(int32(5)),
		SyncPolicy:      permissionV1.RoleMetadata_AUTO.Enum(),
		Scope:           permissionV1.RoleMetadata_TENANT.Enum(),
	}), "Upsert 的冲突目标 (role_id) 不匹配唯一索引 (tenant_id, role_id)，应被数据库拒绝")

	// 被拒绝的 Upsert 不应留下任何行
	exist, err := repo.IsExistByRoleID(ctx, roleID)
	require.NoError(t, err)
	require.False(t, exist, "被拒绝的 Upsert 不应落任何行")
	rowCount, err := repo.entClient.Client().RoleMetadata.Query().Count(ctx)
	require.NoError(t, err)
	require.Zero(t, rowCount, "sys_role_metadata 应无任何记录")
}
