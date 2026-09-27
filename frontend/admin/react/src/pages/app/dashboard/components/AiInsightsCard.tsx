import { useState } from 'react';
import { Button, Skeleton, Tag, Typography, App } from 'antd';
import { RobotOutlined, ThunderboltOutlined } from '@ant-design/icons';
import { useMutation } from '@tanstack/react-query';
import { apiClient } from '@/api/client';
import { useI18n } from '@/core/i18n';
import type { AiInsightAlert } from '@/api/generated/admin/service/v1';

/** 严重度 → 左边条/标签色（语义一致：红=高、橙=中、蓝=低）。 */
const SEVERITY_STYLE: Record<string, { bar: string; tagColor: string }> = {
  HIGH: { bar: 'bg-red-500', tagColor: 'red' },
  MEDIUM: { bar: 'bg-orange-500', tagColor: 'orange' },
  LOW: { bar: 'bg-blue-500', tagColor: 'blue' },
};

/**
 * 安全与异常洞察卡片（dashboard）：
 * 规则预筛审计明细里的行为模式（深夜操作 / 操作失败集中 / 疑似口令尝试 / 敏感操作）
 * 生成结构化告警（确定性事实），LLM 仅生成总体评估措辞。
 * 平台用户专属（后端对租户返回 403——明细日志为全平台数据且会外发到模型端点）。
 */
const AiInsightsCard = () => {
  const { t } = useI18n('dashboard');
  const { message } = App.useApp();
  const [alerts, setAlerts] = useState<AiInsightAlert[] | null>(null);
  const [summary, setSummary] = useState('');

  const insightsMutation = useMutation({
    mutationFn: () => apiClient.dashboardService.GetAiInsights({}),
    onSuccess: (resp) => {
      setAlerts(resp.alerts ?? []);
      setSummary(resp.summary ?? '');
    },
    onError: (error: Error) => {
      console.error('dashboard ai insights failed:', error);
      message.error(error.message || t('aiInsights.failed'));
    },
  });

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

      {/* 头部 */}
      <div className="relative mb-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-blue-500/12 text-blue-400">
            <RobotOutlined style={{ fontSize: 18 }} />
          </div>
          <div>
            <div className="text-sm font-semibold text-[color:var(--ant-color-text)]">
              {t('aiInsights.title')}
            </div>
            <div className="text-xs text-[color:var(--ant-color-text-tertiary)]">
              {t('aiInsights.subtitle')}
            </div>
          </div>
        </div>
        <Button
          type="primary"
          icon={<ThunderboltOutlined />}
          loading={insightsMutation.isPending}
          onClick={() => insightsMutation.mutate()}
        >
          {alerts !== null ? t('aiInsights.regenerate') : t('aiInsights.generate')}
        </Button>
      </div>

      {insightsMutation.isPending && alerts === null ? (
        <Skeleton active paragraph={{ rows: 4 }} />
      ) : alerts === null ? (
        <Typography.Text type="secondary">{t('aiInsights.empty')}</Typography.Text>
      ) : alerts.length === 0 ? (
        <Typography.Text type="secondary">{t('aiInsights.noAnomaly')}</Typography.Text>
      ) : (
        <>
          {/* 告警条目：严重度左边条 + 标签 + 标题 + 明细 */}
          <div className="relative space-y-3">
            {alerts.map((alert, i) => {
              const style = SEVERITY_STYLE[alert.severity ?? ''] ?? SEVERITY_STYLE.LOW!;
              return (
                <div
                  key={i}
                  className="flex items-stretch gap-3 rounded-lg border border-solid border-white/8 p-3"
                >
                  <span className={`w-1 shrink-0 rounded-full ${style.bar}`} />
                  <div className="min-w-0">
                    <div className="mb-0.5 flex flex-wrap items-center gap-2">
                      <Tag color={style.tagColor} className="!m-0">
                        {alert.severity}
                      </Tag>
                      <span className="text-sm font-medium text-[color:var(--ant-color-text)]">
                        {alert.title}
                      </span>
                    </div>
                    <div className="text-xs leading-relaxed text-[color:var(--ant-color-text-secondary)]">
                      {alert.detail}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>

          {/* LLM 总体评估（告警为确定性事实，此处仅为措辞归纳） */}
          {summary && (
            <div className="mt-3 border-t border-solid border-white/8 pt-3">
              <Typography.Text type="secondary" className="!text-xs">
                {t('aiInsights.assessment')}
              </Typography.Text>
              <div className="mt-1 text-sm leading-relaxed text-[color:var(--ant-color-text)]">
                {summary}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};

export default AiInsightsCard;
