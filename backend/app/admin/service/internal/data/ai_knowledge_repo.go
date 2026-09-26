package data

import (
	"context"
	"time"

	"entgo.io/ent/dialect/sql"
	"github.com/tx7do/kratos-bootstrap/bootstrap"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"

	paginationV1 "github.com/tx7do/go-crud/api/gen/go/pagination/v1"
	entCrud "github.com/tx7do/go-crud/entgo"

	"github.com/tx7do/go-utils/copierutil"
	"github.com/tx7do/go-utils/mapper"

	appViewer "go-wind-admin/pkg/entgo/viewer"

	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/app/admin/service/internal/data/ent/aidoc"
	"go-wind-admin/app/admin/service/internal/data/ent/aiknowledgebase"
	"go-wind-admin/app/admin/service/internal/data/ent/predicate"

	"google.golang.org/protobuf/types/known/timestamppb"

	aiV1 "go-wind-admin/api/gen/go/ai/service/v1"
)

type AiKnowledgeRepo struct {
	entClient *entCrud.EntClient[*ent.Client]
	log       *bLogger.Helper

	mapper *mapper.CopierMapper[aiV1.AiKnowledgeBase, ent.AiKnowledgeBase]

	repository *entCrud.Repository[
		ent.AiKnowledgeBaseQuery, ent.AiKnowledgeBaseSelect,
		ent.AiKnowledgeBaseCreate, ent.AiKnowledgeBaseCreateBulk,
		ent.AiKnowledgeBaseUpdate, ent.AiKnowledgeBaseUpdateOne,
		ent.AiKnowledgeBaseDelete,
		predicate.AiKnowledgeBase,
		aiV1.AiKnowledgeBase, ent.AiKnowledgeBase,
	]
}

func NewAiKnowledgeRepo(ctx *bootstrap.Context, entClient *entCrud.EntClient[*ent.Client]) *AiKnowledgeRepo {
	repo := &AiKnowledgeRepo{
		log:       ctx.NewLoggerHelper("ai_knowledge/repo/admin-service"),
		entClient: entClient,
		mapper:    mapper.NewCopierMapper[aiV1.AiKnowledgeBase, ent.AiKnowledgeBase](),
	}

	repo.init()

	return repo
}

func (r *AiKnowledgeRepo) init() {
	r.repository = entCrud.NewRepository[
		ent.AiKnowledgeBaseQuery, ent.AiKnowledgeBaseSelect,
		ent.AiKnowledgeBaseCreate, ent.AiKnowledgeBaseCreateBulk,
		ent.AiKnowledgeBaseUpdate, ent.AiKnowledgeBaseUpdateOne,
		ent.AiKnowledgeBaseDelete,
		predicate.AiKnowledgeBase,
		aiV1.AiKnowledgeBase, ent.AiKnowledgeBase,
	](r.mapper)

	r.mapper.AppendConverters(copierutil.NewTimeStringConverterPair())
	r.mapper.AppendConverters(copierutil.NewTimeTimestamppbConverterPair())
}

// ── 知识库 CRUD（泛型仓库标准形态） ─────────────────────────────────

func (r *AiKnowledgeRepo) Count(ctx context.Context, req *paginationV1.PagingRequest) (uint64, error) {
	builder := r.entClient.Client().AiKnowledgeBase.Query()

	whereSelectors, _, _ := r.repository.BuildListSelectorWithPaging(builder, req)
	if len(whereSelectors) != 0 {
		builder.Modify(whereSelectors...)
	}

	count, err := builder.Count(ctx)
	if err != nil {
		r.log.Errorf(ctx, "query ai knowledge base count failed: %s", err.Error())
		return 0, aiV1.ErrorInternalServerError("query ai knowledge base count failed")
	}
	return uint64(count), nil
}

func (r *AiKnowledgeRepo) List(ctx context.Context, req *paginationV1.PagingRequest) ([]*aiV1.AiKnowledgeBase, uint64, error) {
	if req == nil {
		return nil, 0, aiV1.ErrorBadRequest("invalid parameter")
	}

	builder := r.entClient.Client().AiKnowledgeBase.Query()

	ret, err := r.repository.ListWithPaging(ctx, builder, builder.Clone(), req)
	if err != nil {
		return nil, 0, err
	}

	return ret.Items, ret.Total, nil
}

func (r *AiKnowledgeRepo) Get(ctx context.Context, req *aiV1.GetAiKnowledgeBaseRequest) (*aiV1.AiKnowledgeBase, error) {
	if req == nil {
		return nil, aiV1.ErrorBadRequest("invalid parameter")
	}

	builder := r.entClient.Client().AiKnowledgeBase.Query()

	var whereCond []func(s *sql.Selector)
	switch req.QueryBy.(type) {
	default:
	case *aiV1.GetAiKnowledgeBaseRequest_Id:
		whereCond = append(whereCond, aiknowledgebase.IDEQ(req.GetId()))
	}

	return r.repository.Get(ctx, builder, req.GetViewMask(), whereCond...)
}

func (r *AiKnowledgeRepo) Create(ctx context.Context, data *aiV1.AiKnowledgeBase) (*ent.AiKnowledgeBase, error) {
	if data == nil {
		return nil, aiV1.ErrorBadRequest("invalid parameter")
	}

	builder := r.entClient.Client().AiKnowledgeBase.Create().
		SetNillableName(data.Name).
		SetNillableDescription(data.Description).
		SetNillableProviderID(data.ProviderId).
		SetNillableEmbeddingModel(data.EmbeddingModel).
		SetNillableUserID(data.UserId).
		SetNillableTenantID(data.TenantId).
		SetNillableCreatedBy(data.CreatedBy).
		SetCreatedAt(time.Now())

	entity, err := builder.Save(ctx)
	if err != nil {
		r.log.Errorf(ctx, "insert ai knowledge base failed: %s", err.Error())
		return nil, aiV1.ErrorInternalServerError("insert ai knowledge base failed")
	}
	return entity, nil
}

func (r *AiKnowledgeRepo) Update(ctx context.Context, req *aiV1.UpdateAiKnowledgeBaseRequest) error {
	if req == nil || req.Data == nil {
		return aiV1.ErrorBadRequest("invalid parameter")
	}
	if req.GetId() == 0 {
		return aiV1.ErrorBadRequest("id is required")
	}

	builder := r.entClient.Client().AiKnowledgeBase.Update()

	if err := r.repository.UpdateX(ctx, builder, req.Data, req.GetUpdateMask(),
		func(dto *aiV1.AiKnowledgeBase) {
			builder.
				SetNillableName(req.Data.Name).
				SetNillableDescription(req.Data.Description).
				SetNillableProviderID(req.Data.ProviderId).
				SetNillableEmbeddingModel(req.Data.EmbeddingModel).
				SetNillableUpdatedBy(req.Data.UpdatedBy).
				SetUpdatedAt(time.Now())
		},
		func(s *sql.Selector) {
			s.Where(sql.EQ(aiknowledgebase.FieldID, req.GetId()))
		},
	); err != nil {
		r.log.Errorf(ctx, "update ai knowledge base failed: %s", err.Error())
		return err
	}
	return nil
}

// Delete 删除知识库；docs 边 Cascade 级联删文档行，文档的 chunks 由 doc 侧 Cascade 删除。
func (r *AiKnowledgeRepo) Delete(ctx context.Context, id uint32) error {
	if _, err := r.entClient.Client().AiKnowledgeBase.Delete().
		Where(aiknowledgebase.IDEQ(id)).
		Exec(ctx); err != nil {
		r.log.Errorf(ctx, "delete ai knowledge base failed: %s", err.Error())
		return aiV1.ErrorInternalServerError("delete ai knowledge base failed")
	}
	return nil
}

func (r *AiKnowledgeRepo) GetEntityByID(ctx context.Context, id uint32) (*ent.AiKnowledgeBase, error) {
	entity, err := r.entClient.Client().AiKnowledgeBase.Query().
		Where(aiknowledgebase.IDEQ(id)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, aiV1.ErrorNotFound("ai knowledge base not found")
		}
		r.log.Errorf(ctx, "query ai knowledge base failed: %s", err.Error())
		return nil, aiV1.ErrorInternalServerError("query ai knowledge base failed")
	}
	return entity, nil
}

func (r *AiKnowledgeRepo) ToDTO(entity *ent.AiKnowledgeBase) *aiV1.AiKnowledgeBase {
	if entity == nil {
		return nil
	}
	return r.mapper.ToDTO(entity)
}

// ── 文档（手写 ent 调用：固定 base_id 查询，不走通用 filter） ────────

func (r *AiKnowledgeRepo) CreateDoc(ctx context.Context, data *aiV1.AiDoc) (*ent.AiDoc, error) {
	if data == nil {
		return nil, aiV1.ErrorBadRequest("invalid parameter")
	}
	entity, err := r.entClient.Client().AiDoc.Create().
		SetNillableBaseID(data.BaseId).
		SetNillableName(data.Name).
		SetNillableChunkCount(data.ChunkCount).
		SetNillableStatus(data.Status).
		SetNillableErrorMessage(data.ErrorMessage).
		SetNillableUserID(data.UserId).
		SetNillableTenantID(data.TenantId).
		SetCreatedAt(time.Now()).
		Save(ctx)
	if err != nil {
		r.log.Errorf(ctx, "insert ai doc failed: %s", err.Error())
		return nil, aiV1.ErrorInternalServerError("insert ai doc failed")
	}
	return entity, nil
}

func (r *AiKnowledgeRepo) UpdateDocStatus(ctx context.Context, id uint32, status string, chunkCount uint32, errMsg string) error {
	builder := r.entClient.Client().AiDoc.UpdateOneID(id).
		SetNillableStatus(&status).
		SetChunkCount(chunkCount)
	if errMsg != "" {
		builder.SetErrorMessage(errMsg)
	}
	if err := builder.Exec(ctx); err != nil {
		r.log.Errorf(ctx, "update ai doc status failed: %s", err.Error())
		return aiV1.ErrorInternalServerError("update ai doc failed")
	}
	return nil
}

func (r *AiKnowledgeRepo) ListDocs(ctx context.Context, baseId uint32) ([]*ent.AiDoc, error) {
	return r.entClient.Client().AiDoc.Query().
		Where(aidoc.BaseIDEQ(baseId)).
		Order(ent.Asc(aidoc.FieldID)).
		All(ctx)
}

func (r *AiKnowledgeRepo) GetDocByID(ctx context.Context, id uint32) (*ent.AiDoc, error) {
	entity, err := r.entClient.Client().AiDoc.Query().
		Where(aidoc.IDEQ(id)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, aiV1.ErrorNotFound("ai doc not found")
		}
		r.log.Errorf(ctx, "query ai doc failed: %s", err.Error())
		return nil, aiV1.ErrorInternalServerError("query ai doc failed")
	}
	return entity, nil
}

func (r *AiKnowledgeRepo) DeleteDoc(ctx context.Context, id uint32) error {
	if _, err := r.entClient.Client().AiDoc.Delete().
		Where(aidoc.IDEQ(id)).
		Exec(ctx); err != nil {
		r.log.Errorf(ctx, "delete ai doc failed: %s", err.Error())
		return aiV1.ErrorInternalServerError("delete ai doc failed")
	}
	return nil
}

func (r *AiKnowledgeRepo) ToDocDTO(entity *ent.AiDoc) *aiV1.AiDoc {
	if entity == nil {
		return nil
	}
	return &aiV1.AiDoc{
		Id:           &entity.ID,
		BaseId:       entity.BaseID,
		Name:         entity.Name,
		ChunkCount:   entity.ChunkCount,
		Status:       entity.Status,
		ErrorMessage: entity.ErrorMessage,
		UserId:       entity.UserID,
		CreatedAt:    timestamppbPtr(entity.CreatedAt),
	}
}

// ── 切片（原生 SQL：pgvector 列不在 ent schema 内） ─────────────────

// InsertChunks 批量写入切片与向量。vecLiteral 为 pgvector 字面量 '[0.1,0.2,...]'。
func (r *AiKnowledgeRepo) InsertChunks(ctx context.Context, tenantId uint32, docId uint32, chunks []string, vectors []string) error {
	if len(chunks) != len(vectors) {
		return aiV1.ErrorInternalServerError("chunk/vector length mismatch")
	}
	db := r.entClient.DB()
	for i, content := range chunks {
		_, err := db.ExecContext(ctx,
			`INSERT INTO sys_ai_chunks (created_at, tenant_id, doc_id, content, chunk_index, embedding)
			 VALUES (NOW(), $1, $2, $3, $4, $5::vector)`,
			tenantId, docId, content, uint32(i), vectors[i],
		)
		if err != nil {
			r.log.Errorf(ctx, "insert ai chunk failed: index=%d: %s", i, err.Error())
			return aiV1.ErrorInternalServerError("insert ai chunk failed")
		}
	}
	return nil
}

// ChunkHit 检索命中。
type ChunkHit struct {
	Content    string  `json:"content"`
	ChunkIndex uint32  `json:"chunk_index"`
	DocID      uint32  `json:"doc_id"`
	Score      float64 `json:"score"`
}

// SearchChunks 余弦相似度检索：限定租户 + 知识库，取 topK。
func (r *AiKnowledgeRepo) SearchChunks(ctx context.Context, tenantId, baseId uint32, vecLiteral string, topK int) ([]*ChunkHit, error) {
	if topK <= 0 {
		topK = 3
	}
	db := r.entClient.DB()
	rows, err := db.QueryContext(ctx,
		`SELECT c.content, c.chunk_index, c.doc_id, 1 - (c.embedding <=> $1::vector) AS score
		 FROM sys_ai_chunks c
		 JOIN sys_ai_docs d ON d.id = c.doc_id
		 WHERE d.base_id = $2 AND ($3 = 0 OR c.tenant_id = $3)
		 ORDER BY c.embedding <=> $1::vector
		 LIMIT $4`,
		vecLiteral, baseId, tenantId, topK,
	)
	if err != nil {
		r.log.Errorf(ctx, "search ai chunks failed: %s", err.Error())
		return nil, aiV1.ErrorInternalServerError("search ai chunks failed")
	}
	defer func() { _ = rows.Close() }()

	hits := make([]*ChunkHit, 0, topK)
	for rows.Next() {
		var hit ChunkHit
		if err := rows.Scan(&hit.Content, &hit.ChunkIndex, &hit.DocID, &hit.Score); err != nil {
			r.log.Errorf(ctx, "scan ai chunk hit failed: %s", err.Error())
			return nil, aiV1.ErrorInternalServerError("scan ai chunk failed")
		}
		hits = append(hits, &hit)
	}
	return hits, rows.Err()
}

// ── 向量列启动迁移（幂等） ─────────────────────────────────────────

// MigrateVectorColumn 启动期补建 pgvector 扩展与 embedding 列。
// 扩展需要超级用户；失败只记日志不阻断服务（RAG 功能降级，其余功能不受影响）。
func (r *AiKnowledgeRepo) MigrateVectorColumn(ctx context.Context) error {
	db := r.entClient.DB()
	if _, err := db.ExecContext(ctx, `CREATE EXTENSION IF NOT EXISTS vector`); err != nil {
		r.log.Errorf(ctx, "create pgvector extension failed (rag disabled): %s", err.Error())
		return err
	}
	if _, err := db.ExecContext(ctx,
		`ALTER TABLE sys_ai_chunks ADD COLUMN IF NOT EXISTS embedding vector`); err != nil {
		r.log.Errorf(ctx, "add embedding column failed: %s", err.Error())
		return err
	}
	r.log.Infof(ctx, "pgvector ready: sys_ai_chunks.embedding")
	return nil
}

// timestamppbPtr time → timestamppb（nil 安全）。
func timestamppbPtr(t *time.Time) *timestamppb.Timestamp {
	if t == nil {
		return nil
	}
	return timestamppb.New(*t)
}

// FillDocCounts 一次聚合填充各知识库的文档数（List 读视图）。
func (r *AiKnowledgeRepo) FillDocCounts(ctx context.Context, bases []*aiV1.AiKnowledgeBase) {
	if len(bases) == 0 {
		return
	}
	ids := make([]uint32, 0, len(bases))
	for _, b := range bases {
		if b.Id != nil {
			ids = append(ids, *b.Id)
		}
	}
	if len(ids) == 0 {
		return
	}

	var rows []struct {
		BaseID uint32 `sql:"base_id"`
		Count  uint64 `sql:"count"`
	}
	sysCtx := appViewer.NewSystemViewerContext(ctx)
	if err := r.entClient.Client().AiDoc.Query().
		Where(aidoc.BaseIDIn(ids...)).
		GroupBy(aidoc.FieldBaseID).
		Aggregate(ent.As(ent.Count(), "count")).
		Scan(sysCtx, &rows); err != nil {
		r.log.Errorf(ctx, "count ai docs failed: %s", err.Error())
		return
	}
	counts := make(map[uint32]uint32, len(rows))
	for _, row := range rows {
		counts[row.BaseID] = uint32(row.Count)
	}
	for _, b := range bases {
		if b.Id != nil {
			c := counts[*b.Id]
			b.DocCount = &c
		}
	}
}

// ── 重索引（批量重算 embedding） ─────────────────────────────────────

// ChunkRef 重索引用的切片引用。
type ChunkRef struct {
	ID      uint32
	Content string
}

// ListBases 全量知识库（系统查看器；重索引任务无请求上下文）。
func (r *AiKnowledgeRepo) ListBases(ctx context.Context) ([]*ent.AiKnowledgeBase, error) {
	sysCtx := appViewer.NewSystemViewerContext(ctx)
	return r.entClient.Client().AiKnowledgeBase.Query().All(sysCtx)
}

// ListChunkRefsByBase 取知识库下全部切片的 id+content（原生 SQL：chunks 表不在泛型仓库域）。
func (r *AiKnowledgeRepo) ListChunkRefsByBase(ctx context.Context, baseId uint32) ([]ChunkRef, error) {
	rows, err := r.entClient.DB().QueryContext(ctx,
		`SELECT c.id, c.content FROM sys_ai_chunks c
		 JOIN sys_ai_docs d ON d.id = c.doc_id
		 WHERE d.base_id = $1
		 ORDER BY c.id`,
		baseId,
	)
	if err != nil {
		r.log.Errorf(ctx, "list chunks for reindex failed: %s", err.Error())
		return nil, aiV1.ErrorInternalServerError("list chunks failed")
	}
	defer func() { _ = rows.Close() }()

	refs := make([]ChunkRef, 0, 64)
	for rows.Next() {
		var ref ChunkRef
		var content []byte
		if err := rows.Scan(&ref.ID, &content); err != nil {
			r.log.Errorf(ctx, "scan chunk ref failed: %s", err.Error())
			return nil, aiV1.ErrorInternalServerError("scan chunks failed")
		}
		ref.Content = string(content)
		refs = append(refs, ref)
	}
	return refs, rows.Err()
}

// UpdateChunkEmbedding 就地更新切片向量。
func (r *AiKnowledgeRepo) UpdateChunkEmbedding(ctx context.Context, chunkId uint32, vecLiteral string) error {
	_, err := r.entClient.DB().ExecContext(ctx,
		`UPDATE sys_ai_chunks SET embedding = $1::vector WHERE id = $2`,
		vecLiteral, chunkId,
	)
	if err != nil {
		r.log.Errorf(ctx, "update chunk embedding failed: chunk=%d: %s", chunkId, err.Error())
		return aiV1.ErrorInternalServerError("update chunk embedding failed")
	}
	return nil
}
