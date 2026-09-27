import { useState } from 'react';
import { Button, Skeleton, Typography, App } from 'antd';
import { RobotOutlined, ThunderboltOutlined } from '@ant-design/icons';
import { useMutation } from '@tanstack/react-query';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { apiClient } from '@/api/client';
import { useI18n } from '@/core/i18n';

/**
 * AI 解读卡片：把当日指标喂给默认模型生成一段运营解读（平台用户专属，
 * 后端对租户用户返回 403——解读会把统计数据外发到模型端点）。
 * 按需生成（点按钮才调 LLM），不随页面自动加载。
 *
 * 样式与 StatsCard 同一设计语言：token 背景 + white/8 边框 + 主色 chip/微光，
 * 不用裸色值（亮暗两模式由 token 自适应）；按钮用实心 primary（ghost 在暗色下不可见）。
 */
const AiInsightsCard = () => {
  const { t } = useI18n('dashboard');
  const { message } = App.useApp();
  const [summary, setSummary] = useState('');

  const insightsMutation = useMutation({
    mutationFn: () => apiClient.dashboardService.GetAiInsights({}),
    onSuccess: (resp) => setSummary(resp.summary ?? ''),
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
      <div className="relative flex items-start justify-between">
        <div className="flex shrink-0 items-center justify-center w-11 h-11 rounded-lg ml-0 mr-3 bg-blue-500/12 text-blue-400">
          <RobotOutlined style={{ fontSize: 22 }} />
        </div>
        <div className="flex-1 min-w-0">
          <div className="mb-2 text-sm text-[color:var(--ant-color-text-secondary)]">
            {t('aiInsights.title')}
          </div>
          {insightsMutation.isPending ? (
            <Skeleton active paragraph={{ rows: 2 }} title={false} />
          ) : summary ? (
            <div className="prose prose-sm max-w-none break-words text-[color:var(--ant-color-text)] [&_li]:m-0 [&_p]:mb-1 [&_ul]:my-1 [&_ul]:pl-5">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>{summary}</ReactMarkdown>
            </div>
          ) : (
            <Typography.Text type="secondary">{t('aiInsights.empty')}</Typography.Text>
          )}
        </div>
        <Button
          className="shrink-0 ml-3"
          type="primary"
          icon={<ThunderboltOutlined />}
          loading={insightsMutation.isPending}
          onClick={() => insightsMutation.mutate()}
        >
          {summary ? t('aiInsights.regenerate') : t('aiInsights.generate')}
        </Button>
      </div>
    </div>
  );
};

export default AiInsightsCard;
