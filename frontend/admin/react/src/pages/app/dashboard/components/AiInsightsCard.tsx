import { useState } from 'react';
import { Button, Skeleton, Typography, App } from 'antd';
import { RobotOutlined, ThunderboltOutlined } from '@ant-design/icons';
import { useMutation } from '@tanstack/react-query';
import ReactECharts from 'echarts-for-react';
import { useI18n } from '@/core/i18n';
import { apiClient } from '@/api/client';
import {
  useDashboardOverview,
  useLoginTrend,
  useLoginStatusDistribution,
  useOperationActionDistribution,
} from '@/api/hooks/dashboard';

/** 动作分布横条的语义色（与 SourceDonutChart 色系一致）。 */
const ACTION_COLORS: Record<string, string> = {
  CREATE: '#3b82f6',
  UPDATE: '#22d3ee',
  DELETE: '#ef4444',
  EXPORT: '#a78bfa',
  ASSIGN: '#34d399',
  IMPORT: '#fbbf24',
  OTHER: '#94a3b8',
};

/**
 * AI 数据报告卡片（dashboard）：
 * 图文结构化展示 —— 指标带 + 7 天登录迷你趋势 + 失败率/动作分布 + AI 洞察要点。
 * 统计图由前端结构化渲染（复用页面已有的 query 缓存），LLM 只负责洞察要点文字；
 * 平台用户专属（后端对租户返回 403——解读会把数据外发到模型端点）。
 */
const AiInsightsCard = () => {
  const { t } = useI18n('dashboard');
  const { message } = App.useApp();
  const [insights, setInsights] = useState<string[]>([]);
  const [generatedAt, setGeneratedAt] = useState('');

  const overviewQuery = useDashboardOverview();
  const trendQuery = useLoginTrend(7);
  const actionDistQuery = useOperationActionDistribution();
  const statusDistQuery = useLoginStatusDistribution();

  const insightsMutation = useMutation({
    mutationFn: () => apiClient.dashboardService.GetAiInsights({}),
    onSuccess: (resp) => {
      setInsights(resp.insights ?? []);
      setGeneratedAt(new Date().toLocaleTimeString());
    },
    onError: (error: Error) => {
      console.error('dashboard ai insights failed:', error);
      message.error(error.message || t('aiInsights.failed'));
    },
  });

  const d = overviewQuery.data;
  const trendPoints = trendQuery.data?.points ?? [];
  const actions = (actionDistQuery.data?.items ?? []).slice().sort((a, b) => (b.count ?? 0) - (a.count ?? 0));
  const actionTotal = actions.reduce((sum, a) => sum + (a.count ?? 0), 0) || 1;
  const statusItems = statusDistQuery.data?.items ?? [];
  const succ = statusItems.find((s) => s.label === 'SUCCESS')?.count ?? 0;
  const fail = statusItems.find((s) => s.label === 'FAILED')?.count ?? 0;
  const failRate = succ + fail > 0 ? Math.round((fail * 100) / (succ + fail)) : 0;

  const miniMetrics = [
    { label: t('stats.todayLoginCount'), value: d?.todayLoginCount ?? 0, color: '#3b82f6' },
    { label: t('stats.todayOperationCount'), value: d?.todayOperationCount ?? 0, color: '#22d3ee' },
    { label: t('stats.userCount'), value: d?.userCount ?? 0, color: '#a78bfa' },
    { label: t('stats.roleCount'), value: d?.roleCount ?? 0, color: '#34d399' },
  ];

  const trendOption = {
    grid: { left: 8, right: 8, top: 6, bottom: 18, containLabel: false },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: trendPoints.map((p) => p.date?.slice(5)),
      axisLine: { lineStyle: { color: 'rgba(128,128,128,0.2)' } },
      axisTick: { show: false },
      axisLabel: { show: false },
    },
    yAxis: { type: 'value', splitLine: { show: false }, axisLabel: { show: false } },
    tooltip: { trigger: 'axis' },
    series: [
      {
        type: 'line',
        smooth: true,
        symbol: 'none',
        data: trendPoints.map((p) => p.count),
        lineStyle: { width: 2, color: '#3b82f6' },
        areaStyle: {
          color: {
            type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              { offset: 0, color: 'rgba(59,130,246,0.35)' },
              { offset: 1, color: 'rgba(59,130,246,0.02)' },
            ],
          },
        },
      },
    ],
  };

  return (
    <div
      className={
        'group relative h-full overflow-hidden rounded-xl border border-white/8 ' +
        'bg-[color:var(--ant-color-bg-container)] p-5 transition-all duration-200 ' +
        'hover:border-white/15 hover:shadow-[0_8px_24px_rgba(0,0,0,0.35)]'
      }
    >
      {/* 左上角主色微光，与 StatsCard 同一装饰语言 */}
      <div
        className={
          'pointer-events-none absolute -top-10 -left-10 h-28 w-28 rounded-full ' +
          'bg-gradient-to-br from-blue-500/10 to-transparent opacity-60 blur-2xl'
        }
      />

      {/* 头部：标题 + 生成时间 + 生成按钮 */}
      <div className="relative mb-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-blue-500/12 text-blue-400">
            <RobotOutlined style={{ fontSize: 18 }} />
          </div>
          <div>
            <div className="text-sm font-semibold text-[color:var(--ant-color-text)]">
              {t('aiInsights.title')}
            </div>
            {generatedAt && (
              <div className="text-xs text-[color:var(--ant-color-text-tertiary)]">
                {t('aiInsights.generatedAt', { time: generatedAt })}
              </div>
            )}
          </div>
        </div>
        <Button
          type="primary"
          icon={<ThunderboltOutlined />}
          loading={insightsMutation.isPending}
          onClick={() => insightsMutation.mutate()}
        >
          {insights.length > 0 ? t('aiInsights.regenerate') : t('aiInsights.generate')}
        </Button>
      </div>

      {insightsMutation.isPending && insights.length === 0 ? (
        <Skeleton active paragraph={{ rows: 5 }} />
      ) : insights.length === 0 ? (
        <Typography.Text type="secondary">{t('aiInsights.empty')}</Typography.Text>
      ) : (
        <>
          {/* 指标带 */}
          <div className="mb-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
            {miniMetrics.map((m) => (
              <div key={m.label} className="rounded-lg border border-solid border-white/8 px-3 py-2">
                <div className="text-xs text-[color:var(--ant-color-text-secondary)]">{m.label}</div>
                <div className="text-xl font-semibold tabular-nums" style={{ color: m.color }}>
                  {m.value.toLocaleString()}
                </div>
              </div>
            ))}
          </div>

          {/* 中排：登录趋势 mini 图 + 失败率/动作分布 */}
          <div className="mb-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
            <div className="rounded-lg border border-solid border-white/8 p-3">
              <div className="mb-1 text-xs text-[color:var(--ant-color-text-secondary)]">
                {t('aiInsights.trend7d')}
              </div>
              <ReactECharts option={trendOption} style={{ height: 110 }} notMerge lazyUpdate />
            </div>
            <div className="rounded-lg border border-solid border-white/8 p-3">
              <div className="mb-2 flex items-baseline justify-between">
                <span className="text-xs text-[color:var(--ant-color-text-secondary)]">
                  {t('aiInsights.failRate')}
                </span>
                <span
                  className="text-lg font-semibold tabular-nums"
                  style={{ color: failRate > 20 ? '#ef4444' : '#34d399' }}
                >
                  {failRate}%
                </span>
              </div>
              <div className="space-y-2">
                {actions.map((a) => {
                  const pct = Math.round(((a.count ?? 0) * 100) / actionTotal);
                  return (
                    <div key={a.label} className="flex items-center gap-2">
                      <span className="w-16 shrink-0 text-xs text-[color:var(--ant-color-text-secondary)]">
                        {a.label}
                      </span>
                      <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-gray-500/15">
                        <div
                          className="h-full rounded-full"
                          style={{ width: `${pct}%`, background: ACTION_COLORS[a.label ?? ''] ?? '#94a3b8' }}
                        />
                      </div>
                      <span className="w-10 shrink-0 text-right text-xs tabular-nums text-[color:var(--ant-color-text-secondary)]">
                        {a.count}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>

          {/* AI 洞察要点 */}
          <div>
            <div className="mb-2 text-xs text-[color:var(--ant-color-text-secondary)]">
              {t('aiInsights.points')}
            </div>
            <div className="space-y-1.5">
              {insights.map((point, i) => (
                <div key={i} className="flex items-start gap-2 text-sm leading-relaxed">
                  <span
                    className="mt-1.5 inline-block h-1.5 w-1.5 shrink-0 rounded-full bg-blue-500"
                  />
                  <span className="text-[color:var(--ant-color-text)]">{point}</span>
                </div>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export default AiInsightsCard;
