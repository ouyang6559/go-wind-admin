import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Button, Empty, Input, Popconfirm, Spin, Tooltip, Typography, App } from 'antd';
import {
  DeleteOutlined,
  PlusOutlined,
  SendOutlined,
  UserOutlined,
  RobotOutlined,
} from '@ant-design/icons';
import { useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import type { aiservicev1_AiConversation, aiservicev1_AiMessage } from '@/api/generated/admin/service/v1';
import { PaginationQuery, globalSSEClient, SSE_EVENT } from '@/core';
import {
  useDeleteAiConversation,
  useListAiConversations,
  useListAiMessages,
  useSendChat,
} from '@/api/hooks/ai-chat';
import ContentContainer from '@/layouts/components/PageContainer/ContentContainer';

interface ChatChunk {
  conversationId?: number;
  seq?: number;
  delta?: string;
}

export default function AiChatPage() {
  const { t } = useTranslation('aiChat');
  const { message: antdMessage } = App.useApp();
  const queryClient = useQueryClient();

  // ── 会话列表 ──────────────────────────────────────────────────────
  const conversationsQuery = useListAiConversations(
    new PaginationQuery({ paging: { page: 1, pageSize: 100 }, orderBy: ['-last_message_at'] }),
  );
  const conversations = useMemo(
    () => (conversationsQuery.data?.items ?? []) as aiservicev1_AiConversation[],
    [conversationsQuery.data],
  );

  const [activeId, setActiveId] = useState<number | undefined>(undefined);

  // 首次加载自动选中最近会话
  useEffect(() => {
    if (activeId === undefined && conversations.length > 0) {
      setActiveId(conversations[0].id);
    }
  }, [conversations, activeId]);

  // ── 消息列表 ──────────────────────────────────────────────────────
  const messagesQuery = useListAiMessages(
    new PaginationQuery({
      paging: { page: 1, pageSize: 200 },
      formValues: activeId ? { conversation_id: activeId } : undefined,
      orderBy: ['id'],
    }),
    { enabled: activeId !== undefined },
  );
  const messages = useMemo(
    () => (messagesQuery.data?.items ?? []) as aiservicev1_AiMessage[],
    [messagesQuery.data],
  );

  // ── 流式累积 ──────────────────────────────────────────────────────
  const [streamingText, setStreamingText] = useState('');
  const scrollRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = useCallback(() => {
    requestAnimationFrame(() => {
      scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: 'smooth' });
    });
  }, []);

  // SSE 订阅：本用户流上的 ai_chat_chunk，按会话归组累积
  useEffect(() => {
    const handler = (data: unknown) => {
      const chunk = data as ChatChunk;
      if (!chunk || typeof chunk !== 'object' || !chunk.conversationId) return;
      setStreamingText((prev) => (chunk.delta ? prev + chunk.delta : prev));
    };
    globalSSEClient.on(SSE_EVENT.AIChatChunk, handler);
    return () => {
      globalSSEClient.off(SSE_EVENT.AIChatChunk, handler);
    };
  }, []);

  // 内容变化时滚到底
  useEffect(() => {
    scrollToBottom();
  }, [messages.length, streamingText, scrollToBottom]);

  // ── 发送 ──────────────────────────────────────────────────────────
  const [input, setInput] = useState('');
  const sendMutation = useSendChat({
    onSuccess: (resp) => {
      setInput('');
      setStreamingText('');
      if (resp.conversation?.id) {
        setActiveId(resp.conversation.id);
      }
      queryClient.invalidateQueries({ queryKey: ['listAiConversations'] });
      queryClient.invalidateQueries({ queryKey: ['listAiMessages'] });
    },
    onError: (error: Error) => {
      setStreamingText('');
      antdMessage.error(error.message || t('chatFailed'));
    },
  });

  const handleSend = () => {
    const content = input.trim();
    if (!content || sendMutation.isPending) return;
    // 先本地渲染用户气泡与占位：响应到达后由列表刷新接管
    setStreamingText('');
    sendMutation.mutate({ conversationId: activeId ?? 0, providerId: 0, content });
  };

  // ── 删除会话 ──────────────────────────────────────────────────────
  const deleteConversationMutation = useDeleteAiConversation({
    onSuccess: () => {
      antdMessage.success(t('deleteSuccess'));
      setStreamingText('');
      
      queryClient.invalidateQueries({ queryKey: ['listAiConversations'] });
      queryClient.invalidateQueries({ queryKey: ['listAiMessages'] });
    },
    onError: (error: Error) => antdMessage.error(error.message || t('fetchFailed')),
  });

  const handleDeleteConversation = (id: number) => {
    deleteConversationMutation.mutate({ id });
    if (activeId === id) {
      setActiveId(undefined);
    }
  };

  const handleNewConversation = () => {
    setActiveId(undefined);
    setStreamingText('');
    messagesQuery.refetch();
  };

  // ── 渲染 ──────────────────────────────────────────────────────────
  const isStreaming = sendMutation.isPending;

  return (
    <ContentContainer heightMode="fixed">
      <div className="flex h-full min-h-0 gap-4">
        {/* 左栏：会话列表 */}
        <div className="flex w-64 shrink-0 flex-col rounded-xl border border-solid border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between px-3 py-2">
            <Typography.Text strong>{t('conversations')}</Typography.Text>
            <Button type="primary" size="small" icon={<PlusOutlined />} onClick={handleNewConversation}>
              {t('newConversation')}
            </Button>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
            {conversations.length === 0 ? (
              <Empty description={t('noConversations')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
            ) : (
              conversations.map((conv) => (
                <div
                  key={conv.id}
                  role="button"
                  tabIndex={0}
                  onClick={() => setActiveId(conv.id)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') setActiveId(conv.id);
                  }}
                  className={`group flex cursor-pointer items-center justify-between rounded-lg px-3 py-2 text-sm ${
                    activeId === conv.id
                      ? 'bg-blue-50 text-blue-600 dark:bg-blue-950 dark:text-blue-300'
                      : 'hover:bg-gray-100 dark:hover:bg-gray-800'
                  }`}
                >
                  <span className="truncate">{conv.title || `#${conv.id}`}</span>
                  <Popconfirm
                    title={t('deleteConversation')}
                    description={t('deleteConversationConfirm')}
                    onConfirm={(e) => {
                      e?.stopPropagation();
                      handleDeleteConversation(conv.id!);
                    }}
                    onCancel={(e) => e?.stopPropagation()}
                  >
                    <Button
                      type="text"
                      size="small"
                      danger
                      className="hidden group-hover:inline-flex"
                      icon={<DeleteOutlined />}
                      onClick={(e) => e.stopPropagation()}
                    />
                  </Popconfirm>
                </div>
              ))
            )}
          </div>
        </div>

        {/* 右栏：消息区 */}
        <div className="flex min-h-0 min-w-0 flex-1 flex-col rounded-xl border border-solid border-gray-200 dark:border-gray-700">
          <div ref={scrollRef} className="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
            {messages.length === 0 && !isStreaming ? (
              <div className="flex h-full flex-col items-center justify-center gap-2 text-gray-400">
                <RobotOutlined className="text-5xl" />
                <Typography.Title level={5} className="!mb-0">
                  {t('emptyTitle')}
                </Typography.Title>
                <Typography.Text type="secondary">{t('emptyDesc')}</Typography.Text>
              </div>
            ) : (
              <>
                {messages.map((msg) => (
                  <MessageBubble key={msg.id} message={msg} />
                ))}
                {isStreaming && (
                  <div className="flex items-start gap-3">
                    <Avatar role="assistant" />
                    <div className="max-w-[75%] rounded-2xl rounded-tl-sm bg-gray-100 px-4 py-2 dark:bg-gray-800">
                      {streamingText ? (
                        <MarkdownContent content={streamingText} />
                      ) : (
                        <Spin size="small" />
                      )}
                      <Typography.Text type="secondary" className="!text-xs">
                        {t('streaming')}
                      </Typography.Text>
                    </div>
                  </div>
                )}
              </>
            )}
          </div>

          {/* 输入区 */}
          <div className="border-t border-solid border-gray-200 p-3 dark:border-gray-700">
            <div className="flex items-end gap-2">
              <Input.TextArea
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder={t('inputPlaceholder')}
                autoSize={{ minRows: 1, maxRows: 5 }}
                onPressEnter={(e) => {
                  if (!e.shiftKey) {
                    e.preventDefault();
                    handleSend();
                  }
                }}
                disabled={isStreaming}
                className="flex-1"
              />
              <Button
                type="primary"
                icon={<SendOutlined />}
                loading={isStreaming}
                onClick={handleSend}
                disabled={!input.trim()}
              >
                {isStreaming ? t('sending') : t('send')}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </ContentContainer>
  );
}

// ── 子组件 ──────────────────────────────────────────────────────────

function Avatar({ role }: { role: string }) {
  const { t } = useTranslation('aiChat');
  return (
    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-500 text-white">
      {role === 'assistant' ? <RobotOutlined /> : <UserOutlined />}
      <span className="sr-only">{role === 'assistant' ? t('assistant') : t('you')}</span>
    </div>
  );
}

function MessageBubble({ message: msg }: { message: aiservicev1_AiMessage }) {
  const { t } = useTranslation('aiChat');
  const isUser = msg.role === 'USER';

  if (isUser) {
    return (
      <div className="flex flex-row-reverse items-start gap-3">
        <Avatar role="user" />
        <div className="max-w-[75%] whitespace-pre-wrap break-words rounded-2xl rounded-tr-sm bg-blue-500 px-4 py-2 text-white">
          {msg.content}
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-start gap-3">
      <Avatar role="assistant" />
      <div className="max-w-[75%] rounded-2xl rounded-tl-sm bg-gray-100 px-4 py-2 dark:bg-gray-800">
        <MarkdownContent content={msg.content ?? ''} />
        {(msg.promptTokens || msg.completionTokens) && (
          <Tooltip title={`${msg.modelName ?? ''} · ${msg.durationMs ?? 0}ms`}>
            <Typography.Text type="secondary" className="!text-xs">
              {t('tokensUsage', { prompt: msg.promptTokens ?? 0, completion: msg.completionTokens ?? 0 })}
            </Typography.Text>
          </Tooltip>
        )}
      </div>
    </div>
  );
}

function MarkdownContent({ content }: { content: string }) {
  return (
    <div className="prose prose-sm max-w-none break-words dark:prose-invert [&_pre]:overflow-x-auto [&_pre]:rounded-lg [&_pre]:bg-gray-900 [&_pre]:p-3 [&_pre]:text-gray-100">
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
    </div>
  );
}
