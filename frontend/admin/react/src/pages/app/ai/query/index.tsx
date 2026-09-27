import { useEffect, useRef, useState } from 'react';
import { Button, Input, Table, Tag, Typography, App } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { SendOutlined, ThunderboltOutlined } from '@ant-design/icons';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useI18n } from '@/core/i18n';
import ContentContainer from '@/layouts/components/PageContainer/ContentContainer';
import { useAskAiQuery, type AskAiQueryParams } from '@/api/hooks/ai-query';
import type { aiservicev1_AiQueryRow } from '@/api/generated/admin/service/v1';

type Round = {
  question: string;
  sql?: string;
  columns?: string[];
  rows?: aiservicev1_AiQueryRow[];
  answer?: string;
  loading: boolean;
};

/** 示例问题（点击直接填入输入框）。 */
const SAMPLE_KEYS = ['sample1', 'sample2', 'sample3'] as const;

/**
 * 智能问数页（平台管理员专属）：自然语言 → 只读 SQL → 结构化结果 → AI 结论。
 * 对话式布局：输入区吸底、消息流自动滚到最新；SQL 原样展示便于核对。
 */
export default function AiQueryPage() {
  const { t, i18n } = useI18n('aiQuery');
  const { message } = App.useApp();
  const [input, setInput] = useState('');
  const [rounds, setRounds] = useState<Round[]>([]);
  const bottomRef = useRef<HTMLDivElement>(null);

  // 新轮次追加/更新时滚到底部
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' });
  }, [rounds]);

  const askMutation = useAskAiQuery({
    onSuccess: (resp) => {
      setRounds((prev) => {
        if (prev.length === 0) return prev;
        const last = prev.length - 1;
        const next = [...prev];
        next[last] = {
          ...next[last]!,
          sql: resp.sql,
          columns: resp.columns ?? [],
          rows: resp.rows ?? [],
          answer: resp.answer ?? '',
          loading: false,
        };
        return next;
      });
    },
    onError: (error: Error) => {
      setRounds((prev) => {
        if (prev.length === 0) return prev;
        const last = prev.length - 1;
        const next = [...prev];
        next[last] = { ...next[last]!, loading: false };
        return next;
      });
      message.error(error.message || t('failed'));
    },
  });

  const handleAsk = () => {
    const question = input.trim();
    if (!question || askMutation.isPending) return;
    const params: AskAiQueryParams = { question, lang: i18n.language, withAnswer: true };
    setRounds((prev) => [...prev, { question, loading: true }]);
    setInput('');
    askMutation.mutate(params);
  };

  return (
    <ContentContainer heightMode="auto" scrollable padding="16px">
      <div className="mx-auto flex max-w-4xl flex-col gap-4 pb-2">
        {/* 标题 + 示例问题（仅首轮前展示） */}
        {rounds.length === 0 && (
          <div className="flex flex-col items-center gap-3 py-10 text-center">
            <ThunderboltOutlined className="text-5xl text-blue-500" />
            <Typography.Title level={4} className="!mb-0">
              {t('title')}
            </Typography.Title>
            <Typography.Text type="secondary">{t('emptyDesc')}</Typography.Text>
            <div className="mt-2 flex flex-wrap justify-center gap-2">
              {SAMPLE_KEYS.map((key) => (
                <Button key={key} size="small" onClick={() => setInput(t(`samples.${key}`))}>
                  {t(`samples.${key}`)}
                </Button>
              ))}
            </div>
          </div>
        )}

        {/* 问答轮次 */}
        {rounds.map((round, i) => (
          <div key={i} className="flex flex-col gap-3">
            {/* 用户问题：右侧气泡 */}
            <div className="flex flex-row-reverse items-start gap-3">
              <div className="max-w-[80%] whitespace-pre-wrap break-words rounded-2xl rounded-tr-sm bg-blue-500 px-4 py-2 text-white">
                {round.question}
              </div>
            </div>

            {/* 结果卡片：左侧 */}
            <div className="flex items-start gap-3">
              <div className="min-w-0 flex-1 rounded-xl border border-solid border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-900">
                {round.loading ? (
                  <div className="flex items-center gap-2 text-sm text-gray-400">
                    <span className="inline-block h-2 w-2 animate-pulse rounded-full bg-blue-500" />
                    {t('thinking')}
                  </div>
                ) : (
                  <>
                    {round.sql && (
                      <details className="mb-3" open>
                        <summary className="cursor-pointer text-xs text-gray-400">
                          {t('generatedSql')}
                        </summary>
                        <pre className="mt-2 overflow-x-auto rounded-lg bg-gray-900 p-3 text-xs leading-relaxed text-gray-100">
                          {round.sql}
                        </pre>
                      </details>
                    )}

                    {round.columns && round.columns.length > 0 && (
                      <QueryTable columns={round.columns} rows={round.rows ?? []} />
                    )}
                    {round.sql && (round.rows?.length ?? 0) === 0 && (
                      <Typography.Text type="secondary" className="!text-xs">
                        {t('noRows')}
                      </Typography.Text>
                    )}

                    {round.answer && (
                      <div className="mt-3 rounded-lg border border-solid border-blue-200 bg-blue-50 p-3 text-sm leading-relaxed dark:border-blue-900 dark:bg-blue-950">
                        <Tag color="processing" className="mb-1">
                          {t('answerTag')}
                        </Tag>
                        <div className="markdown-body max-w-none overflow-x-auto break-words [&_li]:m-0 [&_p]:mb-1 [&_table]:w-full [&_table]:border-collapse [&_td]:border [&_td]:border-gray-300 [&_td]:px-2 [&_td]:py-1 [&_th]:border [&_th]:border-gray-300 [&_th]:px-2 [&_th]:py-1 [&_th]:bg-gray-100 dark:[&_td]:border-gray-600 dark:[&_th]:bg-gray-800 dark:[&_th]:border-gray-600">
                          <ReactMarkdown remarkPlugins={[remarkGfm]}>{round.answer}</ReactMarkdown>
                        </div>
                      </div>
                    )}
                  </>
                )}
              </div>
            </div>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>

      {/* 输入区（吸底） */}
      <div className="sticky bottom-0 -mx-4 -mb-4 border-t border-solid border-gray-200 bg-white/95 p-4 backdrop-blur dark:border-gray-700 dark:bg-gray-900/95">
        <div className="mx-auto flex max-w-4xl items-end gap-2">
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onPressEnter={() => handleAsk()}
            placeholder={t('inputPlaceholder')}
            size="large"
            disabled={askMutation.isPending}
            allowClear
          />
          <Button
            type="primary"
            size="large"
            icon={<SendOutlined />}
            loading={askMutation.isPending}
            disabled={!input.trim()}
            onClick={() => handleAsk()}
          >
            {t('ask')}
          </Button>
        </div>
      </div>
    </ContentContainer>
  );
}

/** 查询结果表：以该轮返回的 columns 动态生成列。 */
function QueryTable({
  columns,
  rows,
}: {
  columns: string[];
  rows: aiservicev1_AiQueryRow[];
}) {
  const tableColumns: ColumnsType<{ key: number; cells: string[] }> = columns.map(
    (col, idx) => ({
      title: col,
      dataIndex: ['cells', String(idx)],
      key: col,
      ellipsis: true,
    }),
  );
  const data = rows.map((r, ri) => ({ key: ri, cells: r.values ?? [] }));

  return (
    <Table
      size="small"
      rowKey="key"
      columns={tableColumns}
      dataSource={data}
      pagination={data.length > 10 ? { pageSize: 10, showSizeChanger: false } : false}
      scroll={{ x: 'max-content' }}
    />
  );
}
