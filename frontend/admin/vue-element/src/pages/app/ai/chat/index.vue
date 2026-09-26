<template>
  <div class="app-container chat-container h-full flex flex-1 flex-col">
    <div class="flex h-full min-h-0 gap-4">
      <!-- 左栏：会话列表 -->
      <div class="conversation-panel flex w-64 shrink-0 flex-col">
        <div class="flex items-center justify-between px-3 py-2">
          <span class="font-semibold">{{ t("pages.ai_chat.conversations") }}</span>
          <ElButton type="primary" size="small" @click="handleNewConversation">
            <Icon icon="lucide:plus" class="mr-1" />{{ t("pages.ai_chat.newConversation") }}
          </ElButton>
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
          <div v-if="conversations.length === 0" class="py-8 text-center text-gray-400">
            {{ t("pages.ai_chat.noConversations") }}
          </div>
          <div
            v-for="conv in conversations"
            :key="conv.id"
            class="conversation-item group flex cursor-pointer items-center justify-between rounded-lg px-3 py-2 text-sm"
            :class="{ active: activeId === conv.id }"
            @click="activeId = conv.id"
          >
            <span class="truncate">{{ conv.title || `#${conv.id}` }}</span>
            <ElButton
              type="danger"
              text
              size="small"
              class="hidden group-hover:inline-flex"
              @click.stop="confirmDelete(conv)"
            >
              <Icon icon="lucide:trash-2" />
            </ElButton>
          </div>
        </div>
      </div>

      <!-- 右栏：消息区 -->
      <div class="flex min-h-0 min-w-0 flex-1 flex-col">
        <div ref="scrollRef" class="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
          <div
            v-if="messages.length === 0 && !sending"
            class="flex h-full flex-col items-center justify-center gap-2 text-gray-400"
          >
            <Icon icon="lucide:bot" class="text-5xl" />
            <div class="text-base font-semibold">{{ t("pages.ai_chat.emptyTitle") }}</div>
            <div class="text-sm">{{ t("pages.ai_chat.emptyDesc") }}</div>
          </div>

          <template v-for="msg in messages" :key="msg.id">
            <!-- 用户消息：右侧纯文本气泡 -->
            <div v-if="msg.role === 'USER'" class="flex flex-row-reverse items-start gap-3">
              <div class="avatar flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-500 text-white">
                <Icon icon="lucide:user" />
              </div>
              <div class="max-w-[75%] whitespace-pre-wrap break-words rounded-2xl rounded-tr-sm bg-blue-500 px-4 py-2 text-white">
                {{ msg.content }}
              </div>
            </div>
            <!-- AI 消息：左侧 markdown 渲染 -->
            <div v-else class="flex items-start gap-3">
              <div class="avatar flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-500 text-white">
                <Icon icon="lucide:bot" />
              </div>
              <div class="max-w-[75%] rounded-2xl rounded-tl-sm bg-gray-100 px-4 py-2 dark:bg-gray-800">
                <div class="markdown-body break-words" v-html="renderMarkdown(msg.content || '')" />
                <div
                  v-if="msg.promptTokens || msg.completionTokens"
                  class="mt-1 text-xs text-gray-400"
                  :title="`${msg.modelName || ''} · ${msg.durationMs || 0}ms`"
                >
                  {{ t("pages.ai_chat.tokensUsage", { prompt: msg.promptTokens ?? 0, completion: msg.completionTokens ?? 0 }) }}
                </div>
              </div>
            </div>
          </template>

          <!-- 流式占位气泡 -->
          <div v-if="sending" class="flex items-start gap-3">
            <div class="avatar flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-500 text-white">
              <Icon icon="lucide:bot" />
            </div>
            <div class="max-w-[75%] rounded-2xl rounded-tl-sm bg-gray-100 px-4 py-2 dark:bg-gray-800">
              <div v-if="streamingText" class="markdown-body break-words" v-html="renderMarkdown(streamingText)" />
              <div v-else class="py-1"><ElIcon class="is-loading"><Icon icon="lucide:loader-2" /></ElIcon></div>
              <div class="text-xs text-gray-400">{{ t("pages.ai_chat.streaming") }}</div>
            </div>
          </div>
        </div>

        <!-- 输入区 -->
        <div class="border-t border-solid border-gray-200 p-3 dark:border-gray-700">
          <div class="flex items-end gap-2">
            <ElInput
              v-model="input"
              type="textarea"
              :autosize="{ minRows: 1, maxRows: 5 }"
              :placeholder="t('pages.ai_chat.inputPlaceholder')"
              :disabled="sending"
              @keydown.enter.exact.prevent="handleSend"
            />
            <ElButton type="primary" :loading="sending" :disabled="!input.trim()" @click="handleSend">
              {{ sending ? t("pages.ai_chat.sending") : t("pages.ai_chat.send") }}
            </ElButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { ElButton, ElIcon, ElInput, ElMessage, ElMessageBox } from "element-plus";
import { Icon } from "@iconify/vue";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { useI18n } from "@/core/i18n";
import { PaginationQuery } from "@/core/transport/rest";
import { globalSSEClient, SSE_EVENT } from "@/core/transport/sse";
import type {
  aiservicev1_AiConversation as AiConversation,
  aiservicev1_AiMessage as AiMessage,
} from "@/api/generated/admin/service/v1";
import {
  deleteAiConversation,
  fetchListAiConversations,
  fetchListAiMessages,
  sendAiChat,
} from "@/api/composables";

const { t } = useI18n();

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
      new PaginationQuery({ paging: { page: 1, pageSize: 100 }, orderBy: ["-last_message_at"] }),
    );
    conversations.value = (r.items || []) as AiConversation[];
    if (activeId.value === undefined && conversations.value.length > 0) {
      activeId.value = conversations.value[0]!.id;
    }
  } catch (error) {
    console.error("load ai conversations failed:", error);
    ElMessage.error(t("pages.ai_chat.fetchFailed"));
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
        orderBy: ["id"],
      }),
    );
    messages.value = (r.items || []) as AiMessage[];
    scrollToBottom();
  } catch (error) {
    console.error("load ai messages failed:", error);
    ElMessage.error(t("pages.ai_chat.fetchFailed"));
  }
}

watch(activeId, () => {
  streamingText.value = "";
  loadMessages();
});

// ── 流式 ──────────────────────────────────────────────────────────
const streamingText = ref("");
const scrollRef = ref<HTMLElement>();

function scrollToBottom() {
  nextTick(() => {
    scrollRef.value?.scrollTo({ top: scrollRef.value?.scrollHeight ?? 0, behavior: "smooth" });
  });
}

interface ChatChunk {
  conversationId?: number;
  seq?: number;
  delta?: string;
}

function handleChunk(data: ChatChunk) {
  if (!data || typeof data !== "object" || !data.conversationId) return;
  // 只渲染当前打开会话的片段；其余会话的 chunk 静默丢弃（响应到达后列表刷新）
  if (data.conversationId !== activeId.value) return;
  streamingText.value += data.delta || "";
  scrollToBottom();
}

onMounted(() => {
  loadConversations();
  loadMessages();
  // 必须按回调引用显式注销：组件重挂载反复 on() 会让回调累加（见 useNotice.ts 的注释）
  globalSSEClient.on<ChatChunk>(SSE_EVENT.AIChatChunk, handleChunk);
});

onBeforeUnmount(() => {
  globalSSEClient.off(SSE_EVENT.AIChatChunk, handleChunk);
});

// ── 发送 ──────────────────────────────────────────────────────────
const input = ref("");
const sending = ref(false);

async function handleSend() {
  const content = input.value.trim();
  if (!content || sending.value) return;
  streamingText.value = "";
  sending.value = true;
  scrollToBottom();
  try {
    const resp = await sendAiChat({ conversationId: activeId.value ?? 0, content });
    input.value = "";
    streamingText.value = "";
    if (resp.conversation?.id) {
      const exists = conversations.value.some((c) => c.id === resp.conversation!.id);
      if (!exists) {
        activeId.value = resp.conversation.id;
        await loadConversations();
      }
    }
    await loadMessages();
  } catch (error: any) {
    console.error("send ai chat failed:", error);
    ElMessage.error(error?.message || t("pages.ai_chat.chatFailed"));
  } finally {
    sending.value = false;
  }
}

// ── 删除会话 ──────────────────────────────────────────────────────
async function confirmDelete(conv: AiConversation) {
  const confirmed = await ElMessageBox.confirm(t("pages.ai_chat.deleteConversationConfirm"), t("pages.ai_chat.deleteConversation"), {
    type: "warning",
  }).then(() => true, () => false);
  if (!confirmed) return;
  try {
    await deleteAiConversation(conv.id!);
    ElMessage.success(t("pages.ai_chat.deleteSuccess"));
    if (activeId.value === conv.id) {
      activeId.value = undefined;
      messages.value = [];
    }
    await loadConversations();
  } catch (error: any) {
    console.error("delete ai conversation failed:", error);
    ElMessage.error(error?.message || t("pages.ai_chat.fetchFailed"));
  }
}

function handleNewConversation() {
  activeId.value = undefined;
  messages.value = [];
  streamingText.value = "";
  loadConversations();
}
</script>

<style lang="scss" scoped>
.app-container {
  padding: 20px;
  width: 100%;
  min-width: 0;
}

.chat-container {
  min-height: 0;
}

.conversation-panel {
  border: 1px solid var(--el-border-color-light);
  border-radius: 12px;
  min-height: 0;
}

.conversation-item {
  &:hover {
    background: var(--el-fill-color-light);
  }
  &.active {
    background: var(--el-color-primary-light-9);
    color: var(--el-color-primary);
  }
}

.avatar {
  flex-shrink: 0;
}

.markdown-body {
  :deep(p) {
    margin: 0 0 0.5em;
    &:last-child {
      margin-bottom: 0;
    }
  }
  :deep(pre) {
    background: #0d1117;
    color: #e6edf3;
    border-radius: 8px;
    padding: 12px;
    overflow-x: auto;
    margin: 0.5em 0;
  }
  :deep(code) {
    background: rgba(128, 128, 128, 0.15);
    border-radius: 4px;
    padding: 1px 4px;
  }
  :deep(pre code) {
    background: transparent;
    padding: 0;
  }
  :deep(table) {
    border-collapse: collapse;
    margin: 0.5em 0;
  }
  :deep(th),
  :deep(td) {
    border: 1px solid var(--el-border-color);
    padding: 4px 8px;
  }
}
</style>
