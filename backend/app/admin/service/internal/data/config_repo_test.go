package data

import (
	"context"
	"strings"
	"testing"

	"google.golang.org/genproto/protobuf/field_mask"

	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"github.com/stretchr/testify/require"
	"github.com/tx7do/go-utils/mapper"
	"github.com/tx7do/go-utils/trans"

	configV1 "go-wind-admin/api/gen/go/config/service/v1"
	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/app/admin/service/internal/data/ent/sysconfig"
	"go-wind-admin/app/admin/service/internal/data/enttest"
)

// newConfigRepoSqlite 用 enttest helper 白盒构造一个可直接做 CRUD 的 ConfigRepo
//（同 position_repo_sqlite_test.go 的套路）。
func newConfigRepoSqlite(t *testing.T) *ConfigRepo {
	t.Helper()
	entClient := enttest.NewEntClientForTest(t)
	repo := &ConfigRepo{
		entClient: entClient,
		log:       bLogger.NewHelper(bLogger.NopLogger()),
		mapper:    mapper.NewCopierMapper[configV1.Config, ent.SysConfig](),
		valueTypeConverter: mapper.NewEnumTypeConverter[configV1.Config_ConfigValueType, sysconfig.ValueType](
			configV1.Config_ConfigValueType_name, configV1.Config_ConfigValueType_value,
		),
		cache: make(map[string]sysConfigCacheEntry),
	}
	repo.init()
	return repo
}

func newConfigRepoCtx() context.Context {
	return enttest.NewSystemViewerCtx(context.Background())
}

// TestConfigRepoSqlite_AccessorTypedReads 端到端验证参数读取器：三种类型按声明类型解析，
// 未声明类型（proto 零值守卫路径）落库为默认 STRING。
func TestConfigRepoSqlite_AccessorTypedReads(t *testing.T) {
	repo := newConfigRepoSqlite(t)
	ctx := newConfigRepoCtx()

	err := repo.Create(ctx, &configV1.CreateConfigRequest{
		Data: &configV1.Config{
			Name:      trans.Ptr("是否开启验证码"),
			Key:       trans.Ptr("sys.login.captchaEnabled"),
			Value:     trans.Ptr("true"),
			ValueType: configV1.Config_BOOL.Enum(),
		},
	})
	require.NoError(t, err, "创建 BOOL 参数应成功")
	require.True(t, repo.GetConfigBool(ctx, "sys.login.captchaEnabled", false), "GetConfigBool 应读到 true")

	err = repo.Create(ctx, &configV1.CreateConfigRequest{
		Data: &configV1.Config{
			Name:      trans.Ptr("口令最小长度"),
			Key:       trans.Ptr("sys.password.minLen"),
			Value:     trans.Ptr("12"),
			ValueType: configV1.Config_INT.Enum(),
		},
	})
	require.NoError(t, err)
	require.Equal(t, 12, repo.GetConfigInt(ctx, "sys.password.minLen", 8), "GetConfigInt 应读到 12")

	// 未声明 value_type：Create 走零值守卫跳过设置，落库为 schema 默认 STRING
	err = repo.Create(ctx, &configV1.CreateConfigRequest{
		Data: &configV1.Config{
			Key:   trans.Ptr("sys.demo.plainString"),
			Value: trans.Ptr("hello"),
		},
	})
	require.NoError(t, err, "不声明 value_type 的创建不应触发枚举校验失败")
	require.Equal(t, "hello", repo.GetConfigString(ctx, "sys.demo.plainString", ""), "默认 STRING 类型应按字符串读出")
	require.Equal(t, 8, repo.GetConfigInt(ctx, "sys.demo.plainString", 8), "STRING 参数按 int 读应回退默认值")
}

// TestConfigRepoSqlite_AccessorCacheInvalidation 验证写路径同步失效：
// Update/Delete 后读取器必须立即看到新值/回退默认值，不允许读到陈旧缓存。
func TestConfigRepoSqlite_AccessorCacheInvalidation(t *testing.T) {
	repo := newConfigRepoSqlite(t)
	ctx := newConfigRepoCtx()

	err := repo.Create(ctx, &configV1.CreateConfigRequest{
		Data: &configV1.Config{
			Key:       trans.Ptr("sys.cache.probe"),
			Value:     trans.Ptr("false"),
			ValueType: configV1.Config_BOOL.Enum(),
		},
	})
	require.NoError(t, err)
	require.False(t, repo.GetConfigBool(ctx, "sys.cache.probe", true), "首次读取应落缓存并读到 false")

	// Update（掩码只带 value，模拟前端部分字段编辑）后缓存必须失效
	err = repo.Update(ctx, &configV1.UpdateConfigRequest{
		Id: 1,
		Data: &configV1.Config{
			Value: trans.Ptr("true"),
		},
		UpdateMask: &field_mask.FieldMask{Paths: []string{"value"}},
	})
	require.NoError(t, err)
	require.True(t, repo.GetConfigBool(ctx, "sys.cache.probe", false), "Update 后应读到新值 true（缓存已失效）")

	// Delete 后读取应回退默认值
	err = repo.Delete(ctx, &configV1.DeleteConfigRequest{QueryBy: &configV1.DeleteConfigRequest_Id{Id: 1}})
	require.NoError(t, err)
	require.True(t, repo.GetConfigBool(ctx, "sys.cache.probe", true), "Delete 后应回退默认值 true")
}

// TestConfigRepoSqlite_AccessorMissingKey 验证负缓存与缺省回退。
func TestConfigRepoSqlite_AccessorMissingKey(t *testing.T) {
	repo := newConfigRepoSqlite(t)
	ctx := newConfigRepoCtx()

	require.Equal(t, "def", repo.GetConfigString(ctx, "sys.not.existing", "def"))
	require.Equal(t, "def", repo.GetConfigString(ctx, "sys.not.existing", "def"), "负缓存命中后第二次读取仍回默认")
	require.Equal(t, 7, repo.GetConfigInt(ctx, "sys.not.existing", 7))
}

// TestConfigRepoSqlite_BuiltInDeleteGuard 验证内置参数禁删、非内置可删。
func TestConfigRepoSqlite_BuiltInDeleteGuard(t *testing.T) {
	repo := newConfigRepoSqlite(t)
	ctx := newConfigRepoCtx()

	err := repo.Create(ctx, &configV1.CreateConfigRequest{
		Data: &configV1.Config{
			Key:       trans.Ptr("sys.guard.builtIn"),
			Value:     trans.Ptr("1"),
			ValueType: configV1.Config_INT.Enum(),
			IsBuiltIn: trans.Ptr(true),
		},
	})
	require.NoError(t, err)

	err = repo.Create(ctx, &configV1.CreateConfigRequest{
		Data: &configV1.Config{
			Key:       trans.Ptr("sys.guard.normal"),
			Value:     trans.Ptr("2"),
			ValueType: configV1.Config_INT.Enum(),
		},
	})
	require.NoError(t, err)

	err = repo.Delete(ctx, &configV1.DeleteConfigRequest{QueryBy: &configV1.DeleteConfigRequest_Id{Id: 1}})
	require.Error(t, err, "内置参数删除应被拒绝")
	require.True(t, strings.Contains(err.Error(), "cannot be deleted"), "拒绝原因应说明内置参数禁删")

	err = repo.Delete(ctx, &configV1.DeleteConfigRequest{QueryBy: &configV1.DeleteConfigRequest_Id{Id: 2}})
	require.NoError(t, err, "非内置参数应可删除")

	// 幂等删除：不存在目标视为已删除
	err = repo.Delete(ctx, &configV1.DeleteConfigRequest{QueryBy: &configV1.DeleteConfigRequest_Id{Id: 999}})
	require.NoError(t, err)
}

// TestConfigRepoSqlite_DuplicateKeyRejected 验证唯一键约束映射为 400。
func TestConfigRepoSqlite_DuplicateKeyRejected(t *testing.T) {
	repo := newConfigRepoSqlite(t)
	ctx := newConfigRepoCtx()

	err := repo.Create(ctx, &configV1.CreateConfigRequest{
		Data: &configV1.Config{Key: trans.Ptr("sys.dup.key"), Value: trans.Ptr("a")},
	})
	require.NoError(t, err)

	err = repo.Create(ctx, &configV1.CreateConfigRequest{
		Data: &configV1.Config{Key: trans.Ptr("sys.dup.key"), Value: trans.Ptr("b")},
	})
	require.Error(t, err, "重复键创建应被拒绝")
	require.True(t, strings.Contains(err.Error(), "already exists"), "错误应说明键已存在")
}
