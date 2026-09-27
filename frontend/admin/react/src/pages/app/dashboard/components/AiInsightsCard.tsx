import { useState } from 'react';
import { Button, Typography, App } from 'antd';
import { RobotOutlined } from '@ant-design/icons';
import { useMutation } from '@tanstack/react-query';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { apiClient } from '@/api/client';
import { useI18n } from '@/core/i18n';

/**
 * AI 解读卡片：把当日指标喂给默认模型生成一段运营解读（平台用户专属，
 * 后端对租户用户返回 403——解读会把统计数据外发到模型端点）。
 * 按需生成（点按钮才调 LLM），不随页面自动加载。
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
    <div className="rounded-xl border border-solid border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-900">
      <div className="mb-2 flex items-center justify-between">
        <Typography.Text strong className="inline-flex items-center gap-2">
          <RobotOutlined className="text-blue-500" />
          {t('aiInsights.title')}
        </Typography.Text>
        <Button
          size="small"
          type="primary"
          ghost
          loading={insightsMutation.isPending}
          onClick={() => insightsMutation.mutate()}
        >
          {summary ? t('aiInsights.regenerate') : t('aiInsights.generate')}
        </Button>
      </div>
      {insightsMutation.isPending ? (
        <Typography.Text type="secondary">{t('aiInsights.thinking')}</Typography.Text>
      ) : summary ? (
        <div className="prose prose-sm max-w-none break-words dark:prose-invert [&_li]:m-0 [&_p]:m-0">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{summary}</ReactMarkdown>
        </div>
      ) : (
        <Typography.Text type="secondary">{t('aiInsights.empty')}</Typography.Text>
      )}
    </div>
  );
};

export default AiInsightsCard;
