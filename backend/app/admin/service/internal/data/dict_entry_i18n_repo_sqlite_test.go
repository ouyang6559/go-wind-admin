package data

import (
	"context"
	"testing"

	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"github.com/stretchr/testify/require"
	"github.com/tx7do/go-utils/mapper"

	entCrud "github.com/tx7do/go-crud/entgo"

	dictV1 "go-wind-admin/api/gen/go/dict/service/v1"
	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/app/admin/service/internal/data/ent/dictentry"
	"go-wind-admin/app/admin/service/internal/data/ent/dictentryi18n"
	"go-wind-admin/app/admin/service/internal/data/enttest"
)

// newDictEntryI18nRepoSqlite 在给定 enttest client 上白盒构造 DictEntryI18nRepo，
// 逐字段复刻 NewDictEntryI18nRepo 的 mapper 初始化，再调用 init()。
func newDictEntryI18nRepoSqlite(t *testing.T, entClient *entCrud.EntClient[*ent.Client]) *DictEntryI18nRepo {
	t.Helper()
	repo := &DictEntryI18nRepo{
		entClient: entClient,
		log:       bLogger.NewHelper(bLogger.NopLogger()),
		mapper:    mapper.NewCopierMapper[dictV1.DictEntryI18N, ent.DictEntryI18n](),
	}
	repo.init()
	return repo
}

// TestDictEntryI18nRepoSqlite_UpsertMissingUniqueIndex 固化 Upsert 的现状：
// 其 OnConflictColumns(language_code, entry_id) 要求这两列存在唯一索引，
// 但 schema（sys_dict_entry_i18n）只有 language_code 普通索引，两列无唯一约束，
// SQLite 因此报 "ON CONFLICT clause does not match any PRIMARY KEY or UNIQUE constraint"。
// 该方法当前无生产调用方；若后续补上唯一索引，此断言应翻转为 NoError 并补冲突更新语义。
func TestDictEntryI18nRepoSqlite_UpsertMissingUniqueIndex(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newDictEntryI18nRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	entry, err := entClient.Client().DictEntry.Create().
		SetEntryValue("sqlite_i18n_upsert_value").
		Save(ctx)
	require.NoError(t, err)

	err = repo.Upsert(ctx, 0, 1, entry.ID, "zh-CN", &dictV1.DictEntryI18N{EntryLabel: "标签"})
	require.Error(t, err, "缺 (language_code, entry_id) 唯一索引时 OnConflictColumns 在 SQLite 必然报错")

	cnt, err := entClient.Client().DictEntryI18n.Query().Count(ctx)
	require.NoError(t, err)
	require.Zero(t, cnt, "失败的 Upsert 不应留下任何行")
}

// TestDictEntryI18nRepoSqlite_ListAndGet 验证 ListByEntryID /
// GetByEntryIDAndLangCode 的命中与未命中（行经 ReplaceByEntryID 造出）。
func TestDictEntryI18nRepoSqlite_ListAndGet(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newDictEntryI18nRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	entry, err := entClient.Client().DictEntry.Create().
		SetEntryValue("sqlite_i18n_listget_value").
		Save(ctx)
	require.NoError(t, err)

	// 造行：走生产链路 ReplaceByEntryID（事务内清旧插新）
	tx, err := entClient.Client().Tx(ctx)
	require.NoError(t, err)
	require.NoError(t, repo.ReplaceByEntryID(ctx, tx, 0, 1, entry.ID, map[string]*dictV1.DictEntryI18N{
		"zh-CN": {EntryLabel: "列表查询-标签"},
	}))
	require.NoError(t, tx.Commit())

	// ListByEntryID 命中：返回含 zh-CN 键的映射
	list, err := repo.ListByEntryID(ctx, entry.ID)
	require.NoError(t, err)
	require.Len(t, list, 1)
	require.Contains(t, list, "zh-CN", "翻译映射应包含 zh-CN 键")

	// GetByEntryIDAndLangCode 命中
	hit, err := repo.GetByEntryIDAndLangCode(ctx, entry.ID, "zh-CN")
	require.NoError(t, err, "按存在的语言代码查询应命中")
	require.NotNil(t, hit)

	// GetByEntryIDAndLangCode 未命中：不存在的语言代码
	_, err = repo.GetByEntryIDAndLangCode(ctx, entry.ID, "en-US")
	require.Error(t, err, "不存在的语言代码查询应返回错误")

	// ListByEntryID 未命中：不存在的字典项返回空映射
	missList, err := repo.ListByEntryID(ctx, 424242)
	require.NoError(t, err)
	require.Empty(t, missList, "不存在的字典项应返回空映射")
}

// TestDictEntryI18nRepoSqlite_CleanByEntryIDTx 验证事务版 CleanByEntryID
// 提交后清除指定字典项的翻译行。
func TestDictEntryI18nRepoSqlite_CleanByEntryIDTx(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newDictEntryI18nRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	entry, err := entClient.Client().DictEntry.Create().
		SetEntryValue("sqlite_i18n_clean_tx_value").
		Save(ctx)
	require.NoError(t, err)

	seedTx, err := entClient.Client().Tx(ctx)
	require.NoError(t, err)
	require.NoError(t, repo.ReplaceByEntryID(ctx, seedTx, 0, 1, entry.ID, map[string]*dictV1.DictEntryI18N{
		"zh-CN": {EntryLabel: "待清理-事务"},
	}))
	require.NoError(t, seedTx.Commit())

	rowCount := func() int {
		rows, err := entClient.Client().DictEntryI18n.Query().
			Where(dictentryi18n.HasDictEntryWith(dictentry.IDEQ(entry.ID))).
			All(ctx)
		require.NoError(t, err)
		return len(rows)
	}
	require.Equal(t, 1, rowCount(), "造行后应有 1 条翻译")

	cleanTx, err := entClient.Client().Tx(ctx)
	require.NoError(t, err)
	require.NoError(t, repo.CleanByEntryID(ctx, cleanTx, entry.ID), "事务内 CleanByEntryID 应成功")
	require.NoError(t, cleanTx.Commit())
	require.Zero(t, rowCount(), "事务提交后翻译行应被清除")
}

// TestDictEntryI18nRepoSqlite_CleanByEntryIDs 验证按字典项集合清理：
// 只清指定项，其他项的翻译不受影响。
func TestDictEntryI18nRepoSqlite_CleanByEntryIDs(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newDictEntryI18nRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	entryA, err := entClient.Client().DictEntry.Create().
		SetEntryValue("sqlite_i18n_clean_ids_a").
		Save(ctx)
	require.NoError(t, err)
	entryB, err := entClient.Client().DictEntry.Create().
		SetEntryValue("sqlite_i18n_clean_ids_b").
		Save(ctx)
	require.NoError(t, err)

	for _, entry := range []*ent.DictEntry{entryA, entryB} {
		seedTx, err := entClient.Client().Tx(ctx)
		require.NoError(t, err)
		require.NoError(t, repo.ReplaceByEntryID(ctx, seedTx, 0, 1, entry.ID, map[string]*dictV1.DictEntryI18N{
			"zh-CN": {EntryLabel: "待清理"},
		}))
		require.NoError(t, seedTx.Commit())
	}

	rowCount := func(entryID uint32) int {
		rows, err := entClient.Client().DictEntryI18n.Query().
			Where(dictentryi18n.HasDictEntryWith(dictentry.IDEQ(entryID))).
			All(ctx)
		require.NoError(t, err)
		return len(rows)
	}
	require.Equal(t, 1, rowCount(entryA.ID))
	require.Equal(t, 1, rowCount(entryB.ID))

	require.NoError(t, repo.CleanByEntryIDs(ctx, []uint32{entryA.ID}), "CleanByEntryIDs 应成功")
	require.Zero(t, rowCount(entryA.ID), "指定项的翻译应被清除")
	require.Equal(t, 1, rowCount(entryB.ID), "未指定项的翻译应保留")
}

// TestDictEntryI18nRepoSqlite_Truncate 验证 Truncate 后表内行数归零。
func TestDictEntryI18nRepoSqlite_Truncate(t *testing.T) {
	entClient := enttest.NewEntClientForTest(t)
	repo := newDictEntryI18nRepoSqlite(t, entClient)
	ctx := enttest.NewSystemViewerCtx(context.Background())

	entry, err := entClient.Client().DictEntry.Create().
		SetEntryValue("sqlite_i18n_truncate_value").
		Save(ctx)
	require.NoError(t, err)

	seedTx, err := entClient.Client().Tx(ctx)
	require.NoError(t, err)
	require.NoError(t, repo.ReplaceByEntryID(ctx, seedTx, 0, 1, entry.ID, map[string]*dictV1.DictEntryI18N{
		"zh-CN": {EntryLabel: "待清空"},
	}))
	require.NoError(t, seedTx.Commit())

	existing, err := entClient.Client().DictEntryI18n.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 1, existing, "造行后表内应有 1 条翻译")

	require.NoError(t, repo.Truncate(ctx), "Truncate 应成功")
	total, err := entClient.Client().DictEntryI18n.Query().Count(ctx)
	require.NoError(t, err)
	require.Zero(t, total, "Truncate 后表内行数应为 0")
}
