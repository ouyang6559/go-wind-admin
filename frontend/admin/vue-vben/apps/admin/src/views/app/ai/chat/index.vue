<script lang="ts" setup>
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { $t } from '@vben/locales';
import { LucidePlus, LucideTrash2 } from '@vben/icons';

import { message } from 'ant-design-vue';
import { Icon as Iconify } from '@iconify/vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';

import { PaginationQuery, deleteAiConversation, fetchListAiConversations, fetchListAiKnowledgeBases, fetchListAiMessages, sendAiChat } from '#/api';
import type { aiservicev1_AiKnowledgeBase as AiKnowledgeBase } from '#/api/generated/admin/service/v1';
import { globalSSEClient, SSE_EVENT } from '#/transport/sse';
import type {
  aiservicev1_AiConversation as AiConversation,
  aiservicev1_AiMessage as AiMessage,
} from '#/api/generated/admin/service/v1';

// marked 单行换行按 GFM 处理（聊天场景常见单换行段落）
marked.setOptions({ gfm: true, breaks: true });

function renderMarkdown(content: string): string {
  // AI 回复经 DOMPurify 消毒后再 v-html；LLM 输出不可信，防注入
  return DOMPurify.sanitize(marked.parse(content, { async: false }) as string);
}

// ── 会话 ──────────────────────────────────────────────────────────
const conversations = ref<AiConversation[]>([]);
const activeId = ref<number | undefined>(undefined);

async function loadConversations() {
  try {
    const r = await fetchListAiConversations(
      new PaginationQuery({ paging: { page: 1, pageSize: 100 }, orderBy: ['-last_message_at'] }),
    );
    conversations.value = (r.items || []) as AiConversation[];
    if (activeId.value === undefined && conversations.value.length > 0) {
      activeId.value = conversations.value[0]!.id;
    }
  } catch (error) {
    console.error('load ai conversations failed:', error);
    message.error($t('page.aiChat.fetchFailed'));
  }
}

// ── 消息 ──────────────────────────────────────────────────────────
const messages = ref<AiMessage[]>([]);

async function loadMessages() {
  if (activeId.value === undefined) {
    messages.value = [];
    return;
  }
  try {
    const r = await fetchListAiMessages(
      new PaginationQuery({
        paging: { page: 1, pageSize: 200 },
        formValues: { conversation_id: activeId.value },
        orderBy: ['id'],
      }),
    );
    messages.value = (r.items || []) as AiMessage[];
    scrollToBottom();
  } catch (error) {
    console.error('load ai messages failed:', error);
    message.error($t('page.aiChat.fetchFailed'));
  }
}

watch(activeId, () => {
  streamingText.value = '';
  loadMessages();
});

// ── 流式 ──────────────────────────────────────────────────────────
const streamingText = ref('');
const scrollRef = ref<HTMLElement>();

function scrollToBottom() {
  nextTick(() => {
    scrollRef.value?.scrollTo({ top: scrollRef.value?.scrollHeight ?? 0, behavior: 'smooth' });
  });
}

interface ChatChunk {
  conversationId?: number;
  seq?: number;
  delta?: string;
}

function handleChunk(data: ChatChunk) {
  if (!data || typeof data !== 'object' || !data.conversationId) return;
  // 只渲染当前打开会话的片段；其余会话的 chunk 静默丢弃（响应到达后列表刷新）
  if (data.conversationId !== activeId.value) return;
  streamingText.value += data.delta || '';
  scrollToBottom();
}

onMounted(() => {
  loadConversations();
  loadMessages();
  // 必须按回调引用显式注销：组件重挂载反复 on() 会让回调累加（同 useNotice 的坑）
  globalSSEClient.on<ChatChunk>(SSE_EVENT.AIChatChunk, handleChunk);
});

onBeforeUnmount(() => {
  globalSSEClient.off(SSE_EVENT.AIChatChunk, handleChunk);
});

// ── 知识库选择（RAG：发送时携带 knowledgeBaseId） ─────────────────
const knowledgeBases = ref<AiKnowledgeBase[]>([]);
const knowledgeBaseId = ref<number | undefined>(undefined);

onMounted(() => {
  fetchListAiKnowledgeBases(new PaginationQuery({ paging: { page: 1, pageSize: 100 } }))
    .then((res) => {
      knowledgeBases.value = (res.items || []) as AiKnowledgeBase[];
    })
    .catch((error) => console.error('fetch ai knowledge bases failed:', error));
});

// ── 发送 ──────────────────────────────────────────────────────────
const input = ref('');
const sending = ref(false);

async function handleSend() {
  const content = input.value.trim();
  if (!content || sending.value) return;
  streamingText.value = '';
  sending.value = true;
  scrollToBottom();
  try {
    const resp = await sendAiChat({ conversationId: activeId.value ?? 0, content, knowledgeBaseId: knowledgeBaseId.value ?? 0 });
    input.value = '';
    streamingText.value = '';
    if (resp.conversation?.id) {
      const exists = conversations.value.some((c) => c.id === resp.conversation!.id);
      if (!exists) {
        activeId.value = resp.conversation.id;
        await loadConversations();
      }
    }
    await loadMessages();
  } catch (error) {
    console.error('send ai chat failed:', error);
    message.error(error instanceof Error ? error.message : $t('page.aiChat.chatFailed'));
  } finally {
    sending.value = false;
  }
}

// ── 删除会话 ──────────────────────────────────────────────────────
async function confirmDelete(conv: AiConversation) {
  try {
    await deleteAiConversation(conv.id!);
    message.success($t('page.aiChat.deleteSuccess'));
    if (activeId.value === conv.id) {
      activeId.value = undefined;
      messages.value = [];
    }
    await loadConversations();
  } catch (error) {
    console.error('delete ai conversation failed:', error);
    message.error(error instanceof Error ? error.message : $t('page.aiChat.fetchFailed'));
  }
}

function handleNewConversation() {
  activeId.value = undefined;
  messages.value = [];
  streamingText.value = '';
  loadConversations();
}
</script>

<template>
  <Page auto-content-height>
    <div class="flex h-full min-h-0 gap-4">
      <!-- 左栏：会话列表 -->
      <div class="flex w-64 shrink-0 flex-col rounded-xl border border-solid border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between px-3 py-2">
          <span class="font-semibold">{{ $t('page.aiChat.conversations') }}</span>
          <a-button size="small" type="primary" @click="handleNewConversation">
            <template #icon>
              <LucidePlus />
            </template>
            {{ $t('page.aiChat.newConversation') }}
          </a-button>
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
          <div v-if="conversations.length === 0" class="py-8 text-center text-gray-400">
            {{ $t('page.aiChat.noConversations') }}
          </div>
          <div
            v-for="conv in conversations"
            :key="conv.id"
            class="group flex cursor-pointer items-center justify-between rounded-lg px-3 py-2 text-sm"
            :class="activeId === conv.id
              ? 'bg-primary/10 text-primary'
              : 'hover:bg-gray-100 dark:hover:bg-gray-800'"
            @click="activeId = conv.id"
          >
            <span class="truncate">{{ conv.title || `#${conv.id}` }}</span>
            <a-popconfirm
              :cancel-text="$t('ui.button.cancel')"
              :ok-text="$t('ui.button.ok')"
              :title="$t('page.aiChat.deleteConversationConfirm')"
              @confirm="confirmDelete(conv)"
            >
              <a-button
                class="hidden group-hover:inline-flex"
                danger
                size="small"
                type="text"
                @click.stop
              >
                <LucideTrash2 />
              </a-button>
            </a-popconfirm>
          </div>
        </div>
      </div>

      <!-- 右栏：消息区 -->
      <div class="flex min-h-0 min-w-0 flex-1 flex-col rounded-xl border border-solid border-gray-200 dark:border-gray-700">
        <div ref="scrollRef" class="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
          <div
            v-if="messages.length === 0 && !sending"
            class="flex h-full flex-col items-center justify-center gap-2 text-gray-400"
          >
            <Iconify icon="lucide:bot" class="text-5xl" />
            <div class="text-base font-semibold">{{ $t('page.aiChat.emptyTitle') }}</div>
            <div class="text-sm">{{ $t('page.aiChat.emptyDesc') }}</div>
          </div>

          <template v-for="msg in messages" :key="msg.id">
            <!-- 用户消息：右侧纯文本气泡 -->
            <div v-if="msg.role === 'USER'" class="flex flex-row-reverse items-start gap-3">
              <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-500 text-white">
                <Iconify icon="lucide:user" />
              </div>
              <div class="max-w-[75%] whitespace-pre-wrap break-words rounded-2xl rounded-tr-sm bg-blue-500 px-4 py-2 text-white">
                {{ msg.content }}
              </div>
            </div>
            <!-- AI 消息：左侧 markdown 渲染 -->
            <div v-else class="flex items-start gap-3">
              <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-500 text-white">
                <Iconify icon="lucide:bot" />
              </div>
              <div class="max-w-[75%] rounded-2xl rounded-tl-sm bg-gray-100 px-4 py-2 dark:bg-gray-800">
                <div class="markdown-body break-words" v-html="renderMarkdown(msg.content || '')" />
                <div
                  v-if="msg.promptTokens || msg.completionTokens"
                  class="mt-1 text-xs text-gray-400"
                  :title="`${msg.modelName || ''} · ${msg.durationMs || 0}ms`"
                >
                  {{ $t('page.aiChat.tokensUsage', { prompt: msg.promptTokens ?? 0, completion: msg.completionTokens ?? 0 }) }}
                </div>
              </div>
            </div>
          </template>

          <!-- 流式占位气泡 -->
          <div v-if="sending" class="flex items-start gap-3">
            <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-500 text-white">
              <LucideBot />
            </div>
            <div class="max-w-[75%] rounded-2xl rounded-tl-sm bg-gray-100 px-4 py-2 dark:bg-gray-800">
              <div v-if="streamingText" class="markdown-body break-words" v-html="renderMarkdown(streamingText)" />
              <div v-else class="py-1 text-gray-400">…</div>
              <div class="text-xs text-gray-400">{{ $t('page.aiChat.streaming') }}</div>
            </div>
          </div>
        </div>

        <!-- 输入区 -->
        <div class="border-t border-solid border-gray-200 p-3 dark:border-gray-700">
          <div class="mb-2 flex items-center gap-2">
            <span class="shrink-0 text-xs text-gray-400">{{ $t('page.aiChat.knowledgeBase') }}</span>
            <a-select
              v-model:value="knowledgeBaseId"
              allow-clear
              class="!w-56"
              size="small"
              :placeholder="$t('page.aiChat.knowledgeBasePlaceholder')"
              :options="knowledgeBases.map((kb) => ({ label: kb.name || `#${kb.id}`, value: kb.id }))"
            />
          </div>
          <div class="flex items-end gap-2">
            <a-textarea
              v-model:value="input"
              :auto-size="{ minRows: 1, maxRows: 5 }"
              :disabled="sending"
              :placeholder="$t('page.aiChat.inputPlaceholder')"
              class="flex-1"
              @keydown.enter.exact.prevent="handleSend"
            />
            <a-button
              :disabled="!input.trim()"
              :loading="sending"
              type="primary"
              @click="handleSend"
            >
              {{ sending ? $t('page.aiChat.sending') : $t('page.aiChat.send') }}
            </a-button>
          </div>
        </div>
      </div>
    </div>
  </Page>
</template>

<style scoped>
.markdown-body :deep(p) {
  margin: 0 0 0.5em;
}

.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(pre) {
  background: #0d1117;
  color: #e6edf3;
  border-radius: 8px;
  padding: 12px;
  overflow-x: auto;
  margin: 0.5em 0;
}

.markdown-body :deep(code) {
  background: rgb(128 128 128 / 15%);
  border-radius: 4px;
  padding: 1px 4px;
}

.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
}

.markdown-body :deep(table) {
  border-collapse: collapse;
  margin: 0.5em 0;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid var(--border);
  padding: 4px 8px;
}
</style>
