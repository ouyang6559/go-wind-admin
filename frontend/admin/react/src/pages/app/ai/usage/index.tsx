import { useRef } from 'react';
import type { ProColumns, ActionType } from '@ant-design/pro-components';
import { ProTable } from '@ant-design/pro-components';
import { Progress } from 'antd';
import { useTranslation } from 'react-i18next';
import { PaginationQuery } from '@/core';
import { useGetAiUsageSummary } from '@/api/hooks/ai-usage';
import { fetchListAiUsageLogs } from '@/api/hooks/ai-usage';
import { useProTableScrollY } from '@/hooks/useProTableScrollY';
import ContentContainer from '@/layouts/components/PageContainer/ContentContainer';
import { TABLE } from '@/config/constants';

interface UsageRow {
  id?: number;
  modelName?: string;
  promptTokens?: number;
  completionTokens?: number;
  totalTokens?: number;
  durationMs?: number;
  createdAt?: string;
}

/**
 * AI 用量页：当月汇总（tokens/调用次数/配额进度）+ 用量流水列表。
 * 平台用户见平台侧记录（tenant_id=0）；租户用户由后端租户隔离自动限定本租户。
 */
export default function AiUsagePage() {
  const { t } = useTranslation('aiUsage');
  const actionRef = useRef<ActionType>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const tableScrollY = useProTableScrollY(containerRef);
  const summary = useGetAiUsageSummary();

  const columns: ProColumns<UsageRow>[] = [
    { title: t('model'), dataIndex: 'modelName', minWidth: 160 },
    { title: t('promptTokens'), dataIndex: 'promptTokens', width: 130, render: (v) => Number(v ?? 0).toLocaleString() },
    { title: t('completionTokens'), dataIndex: 'completionTokens', width: 140, render: (v) => Number(v ?? 0).toLocaleString() },
    { title: t('totalTokens'), dataIndex: 'totalTokens', width: 110, render: (v) => Number(v ?? 0).toLocaleString() },
    { title: t('duration'), dataIndex: 'durationMs', width: 100, render: (v) => `${v ?? 0} ms` },
    { title: t('time'), dataIndex: 'createdAt', width: 180, valueType: 'dateTime' },
  ];

  const summaryData = summary.data;
  const quotaPct =
    summaryData?.quotaConfigured && (summaryData.quotaLimit ?? 0) > 0
      ? Math.min(100, Math.round(((summaryData.monthTokens ?? 0) * 100) / (summaryData.quotaLimit || 1)))
      : 0;

  const cards = [
    {
      label: t('summary.monthTokens'),
      value: (summaryData?.monthTokens ?? 0).toLocaleString(),
      color: 'text-blue-500',
    },
    {
      label: t('summary.monthCalls'),
      value: (summaryData?.monthCalls ?? 0).toLocaleString(),
      color: 'text-cyan-500',
    },
  ];

  return (
    <ContentContainer heightMode="fixed" padding="16px" bottomMargin={0}>
      <div ref={containerRef} className="page-container-content">
        {/* 汇总卡 */}
        <div className="mb-4 grid grid-cols-1 gap-4 sm:grid-cols-3">
          {cards.map((c) => (
            <div
              key={c.label}
              className="rounded-xl border border-solid border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-900"
            >
              <div className="mb-2 text-sm text-gray-400">{c.label}</div>
              <div className={`text-2xl font-semibold tabular-nums ${c.color}`}>{c.value}</div>
            </div>
          ))}
          <div className="rounded-xl border border-solid border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-900">
            <div className="mb-2 text-sm text-gray-400">{t('summary.quota')}</div>
            {summaryData?.quotaConfigured ? (
              <>
                <Progress percent={quotaPct} status={quotaPct > 80 ? 'exception' : 'normal'} />
                <div className="mt-1 text-xs text-gray-400">
                  {(summaryData.monthTokens ?? 0).toLocaleString()} / {(summaryData.quotaLimit ?? 0).toLocaleString()}
                </div>
              </>
            ) : (
              <div className="text-2xl font-semibold text-gray-400">∞</div>
            )}
          </div>
        </div>

        {/* 流水列表 */}
        <ProTable<UsageRow>
          actionRef={actionRef}
          rowKey="id"
          search={false}
          scroll={{ y: tableScrollY, x: 800 }}
          request={async (params) => {
            try {
              const { current, pageSize } = params;
              const query = new PaginationQuery({
                paging: { page: current || 1, pageSize: pageSize || TABLE.DEFAULT_PAGE_SIZE },
              });
              const res = await fetchListAiUsageLogs(query);
              return { data: res.items || [], total: res.total || 0, success: true };
            } catch (error) {
              console.error('fetch ai usage logs failed:', error);
              return { data: [], total: 0, success: false };
            }
          }}
          columns={columns}
          toolBarRender={false}
        />
      </div>
    </ContentContainer>
  );
}
