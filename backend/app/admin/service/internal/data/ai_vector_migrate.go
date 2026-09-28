package data

import (
	"context"
	"database/sql"
	"fmt"
	"regexp"
)

// AiEmbeddingDimensions embedding 输出向量维度。
// 与语义检索/知识库向量化请求的 text-embedding-3-small（1536 维）一致；
// 换 embedding 模型必须同步改这里并触发全量重索引，否则写入按维度不符报错。
const AiEmbeddingDimensions = 1536

var vectorTableIdent = regexp.MustCompile(`^[a-z0-9_]+$`)

// EnsureVectorColumnDim 幂等确保 <table>.embedding 存在且为定维
// vector(AiEmbeddingDimensions)：列缺失则补建；维度不匹配（含历史非定维列）
// 则整列重建——旧向量随列丢弃，调用方须提示重索引。定维是硬约束：
// 混维向量会让 <=> 比较在查询期整体报错，宁可写入期 fail-fast。
func EnsureVectorColumnDim(ctx context.Context, db *sql.DB, table string) (rebuilt bool, err error) {
	if !vectorTableIdent.MatchString(table) {
		return false, fmt.Errorf("invalid table name: %s", table)
	}
	if _, err = db.ExecContext(ctx, `CREATE EXTENSION IF NOT EXISTS vector`); err != nil {
		return false, fmt.Errorf("create pgvector extension: %w", err)
	}
	if _, err = db.ExecContext(ctx, fmt.Sprintf(
		"ALTER TABLE %s ADD COLUMN IF NOT EXISTS embedding vector(%d)", table, AiEmbeddingDimensions),
	); err != nil {
		return false, fmt.Errorf("add embedding column: %w", err)
	}

	var typmod int
	if err = db.QueryRowContext(ctx,
		`SELECT a.atttypmod FROM pg_attribute a
		 JOIN pg_type t ON t.oid = a.atttypid
		 WHERE a.attrelid = $1::regclass AND a.attname = 'embedding'
		   AND a.attnum > 0 AND NOT a.attisdropped AND t.typname = 'vector'`,
		table,
	).Scan(&typmod); err != nil {
		return false, fmt.Errorf("read embedding column dimension: %w", err)
	}
	if typmod == AiEmbeddingDimensions {
		return false, nil
	}

	if _, err = db.ExecContext(ctx, fmt.Sprintf(
		"ALTER TABLE %s DROP COLUMN embedding", table)); err != nil {
		return false, fmt.Errorf("drop mismatched embedding column: %w", err)
	}
	if _, err = db.ExecContext(ctx, fmt.Sprintf(
		"ALTER TABLE %s ADD COLUMN embedding vector(%d)", table, AiEmbeddingDimensions),
	); err != nil {
		return false, fmt.Errorf("re-add embedding column: %w", err)
	}
	return true, nil
}
