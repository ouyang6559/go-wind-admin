import { useState } from 'react';
import { Button, Empty, Input, Table, Tag, Typography, App } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { SendOutlined, ThunderboltOutlined } from '@ant-design/icons';
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

/**
 * 智能问数页（平台管理员专属）：自然语言 → 只读 SQL → 结构化结果 → AI 结论。
 * 问答式交互，历史轮次保留在页内；SQL 原样展示便于核对。
 */
export default function AiQueryPage() {
  const { t, i18n } = useI18n('aiQuery');
  const { message } = App.useApp();
  const [input, setInput] = useState('');
  const [rounds, setRounds] = useState<Round[]>([]);

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

  // 动态列：以最新一轮的 columns 为准
  const latest = rounds.length > 0 ? rounds[rounds.length - 1]! : undefined;
  const columns: ColumnsType<{ key: number; cells: string[] }> =
    latest?.columns?.map((col, idx) => ({
      title: col,
      dataIndex: ['cells', String(idx)],
      key: col,
      ellipsis: true,
    })) ?? [];
  const tableData: { key: number; cells: string[] }[] =
    latest?.rows?.map((r, ri) => ({ key: ri, cells: r.values ?? [] })) ?? [];

  return (
    <ContentContainer heightMode="auto" scrollable padding="16px">
      <div className="mx-auto flex max-w-4xl flex-col gap-4">
        {/* 输入区 */}
        <div className="flex items-center gap-2">
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onPressEnter={handleAsk}
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
            onClick={handleAsk}
          >
            {t('ask')}
          </Button>
        </div>

        {rounds.length === 0 && (
          <div className="flex flex-col items-center gap-2 py-16 text-gray-400">
            <ThunderboltOutlined className="text-5xl" />
            <Typography.Title level={4} className="!mb-0">
              {t('title')}
            </Typography.Title>
            <Typography.Text type="secondary">{t('emptyDesc')}</Typography.Text>
          </div>
        )}

        {/* 问答轮次 */}
        {rounds.map((round, i) => (
          <div
            key={i}
            className="rounded-xl border border-solid border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-900"
          >
            {/* 问题 */}
            <Typography.Text strong>{round.question}</Typography.Text>

            {round.loading ? (
              <div className="mt-2 text-sm text-gray-400">{t('thinking')}</div>
            ) : (
              <>
                {/* SQL */}
                {round.sql && (
                  <div className="mt-3">
                    <Tag color="blue">SQL</Tag>
                    <pre className="mt-1 overflow-x-auto rounded-lg bg-gray-900 p-3 text-xs text-gray-100">
                      {round.sql}
                    </pre>
                  </div>
                )}

                {/* 结果表 */}
                {round.columns && round.columns.length > 0 && (
                  <Table
                    className="mt-3"
                    size="small"
                    rowKey={(r) => r.key}
                    columns={columns}
                    dataSource={tableData}
                    pagination={tableData.length > 10 ? { pageSize: 10 } : false}
                    scroll={{ x: 'max-content' }}
                  />
                )}
                {round.sql && (round.rows?.length ?? 0) === 0 && (
                  <Typography.Text type="secondary" className="mt-2 block !text-xs">
                    {t('noRows')}
                  </Typography.Text>
                )}

                {/* AI 结论 */}
                {round.answer && (
                  <div className="mt-3 rounded-lg bg-blue-50 p-3 text-sm leading-relaxed dark:bg-blue-950">
                    <Tag color="processing">{t('answerTag')}</Tag>
                    <span className="whitespace-pre-wrap">{round.answer}</span>
                  </div>
                )}
              </>
            )}
          </div>
        ))}

        {rounds.length > 0 && <div className="h-2" />}
        {rounds.length === 0 && <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} className="hidden" />}
      </div>
    </ContentContainer>
  );
}
