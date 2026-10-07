// 跨包（service 层）测试装配：导出免 bootstrap.Context 的 repo 构造器，
// 字段初始化必须与生产 NewXxxRepo 逐字段一致，改生产构造器须同步此处。
//
// 说明：
//   - 本文件收录 ai_content 全回路测试（ai_content_service_pg_test.go，PG 门控
//     集成）所需的仓库：ai_provider / ai_usage_log（menu 的 ForTest 构造器已在
//     repo_testkit2.go 收录，此处不重复导出）。各构造器与对应 *_repo.go 的生产
//     构造器逐字段对齐（含 mapper/converter 初始化与 init() 调用），唯一差异
//     是 log 一律 bLogger.NewHelper(bLogger.NopLogger())；entClient 由调用方
//     传入（该测试为专用测试库的 PG 客户端）。
package data

import (
	bLogger "github.com/tx7do/kratos-bootstrap/logger"

	entCrud "github.com/tx7do/go-crud/entgo"

	"github.com/tx7do/go-utils/mapper"

	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/app/admin/service/internal/data/ent/aiprovider"

	aiV1 "go-wind-admin/api/gen/go/ai/service/v1"
)

// NewAiProviderRepoForTest 与生产 NewAiProviderRepo 逐字段一致（log 换 NopLogger），并调用 init()。
func NewAiProviderRepoForTest(entClient *entCrud.EntClient[*ent.Client]) *AiProviderRepo {
	repo := &AiProviderRepo{
		log:       bLogger.NewHelper(bLogger.NopLogger()),
		entClient: entClient,
		mapper:    mapper.NewCopierMapper[aiV1.AiProvider, ent.AiProvider](),
		modelTypeConverter: mapper.NewEnumTypeConverter[aiV1.AiProvider_ModelType, aiprovider.ModelType](
			aiV1.AiProvider_ModelType_name,
			aiV1.AiProvider_ModelType_value,
		),
	}

	repo.init()

	return repo
}

// NewAiUsageLogRepoForTest 与生产 NewAiUsageLogRepo 逐字段一致（log 换 NopLogger），并调用 init()。
func NewAiUsageLogRepoForTest(entClient *entCrud.EntClient[*ent.Client]) *AiUsageLogRepo {
	repo := &AiUsageLogRepo{
		log:       bLogger.NewHelper(bLogger.NopLogger()),
		entClient: entClient,
		mapper:    mapper.NewCopierMapper[aiV1.AiUsageLog, ent.AiUsageLog](),
	}

	repo.init()

	return repo
}
