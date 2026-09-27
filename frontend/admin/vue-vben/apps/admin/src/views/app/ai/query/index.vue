<script lang="ts" setup>
import { nextTick, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { $t } from '@vben/locales';
import { message } from 'ant-design-vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';

import { apiClient } from '#/api';

// marked 单行换行按 GFM 处理
marked.setOptions({ gfm: true, breaks: true });

function md(content: string): string {
  // AI 回复经 DOMPurify 消毒后再 v-html；LLM 输出不可信，防注入
  return DOMPurify.sanitize(marked.parse(content, { async: false }) as string);
}

interface Round {
  question: string;
  sql?: string;
  columns?: string[];
  rows?: { cells: string[] }[];
  answer?: string;
  loading: boolean;
}

const sampleKeys = ['sample1', 'sample2', 'sample3'];

const rounds = ref<Round[]>([]);
const input = ref('');
const loading = ref(false);
const scrollRef = ref<HTMLElement>();

function scrollToBottom() {
  nextTick(() => {
    scrollRef.value?.scrollTo({
      top: scrollRef.value?.scrollHeight ?? 0,
      behavior: 'smooth',
    });
  });
}

async function handleAsk(text?: string) {
  const question = (text ?? input.value).trim();
  if (!question || loading.value) return;
  // 携带最近 5 轮历史（问题+SQL+结果摘要），供模型消解追问里的指代
  const history = rounds.value
    .filter((r) => !r.loading && r.sql)
    .slice(-5)
    .map((r) => ({
      question: r.question,
      sql: r.sql || '',
      resultSummary: (r.rows || []).slice(0, 3).map((row) => row.cells.join(' | ')).join('；'),
    }));
  rounds.value.push({ question, loading: true });
  input.value = '';
  loading.value = true;
  scrollToBottom();
  try {
    const lang = $t('page.aiQuery.lang') || 'zh-CN';
    const resp = await apiClient.aiQueryService.Ask({
      question,
      lang,
      withAnswer: true,
      history,
    } as any);
    const last = rounds.value[rounds.value.length - 1];
    if (last) {
      last.sql = resp.sql;
      last.columns = (resp.columns || []) as string[];
      last.rows = (resp.rows || []).map((r) => ({
        cells: (r.values || []) as string[],
      }));
      last.answer = resp.answer || '';
      last.loading = false;
    }
    scrollToBottom();
  } catch (error) {
    console.error('ai query failed:', error);
    message.error(error instanceof Error ? error.message : 'query failed');
    const last = rounds.value[rounds.value.length - 1];
    if (last) last.loading = false;
  } finally {
    loading.value = false;
  }
}

function sampleText(key: string): string {
  const map: Record<string, string> = {
    sample1: $t('page.aiQuery.sample1'),
    sample2: $t('page.aiQuery.sample2'),
    sample3: $t('page.aiQuery.sample3'),
  };
  return map[key] || key;
}
</script>

<template>
  <Page auto-content-height>
    <div class="mx-auto flex h-full max-w-4xl flex-col gap-4">
      <!-- 消息滚动区 -->
      <div
        ref="scrollRef"
        class="min-h-0 flex-1 overflow-y-auto rounded-xl border border-solid border-gray-200 bg-card p-4 dark:border-gray-700"
      >
        <!-- 标题 + 示例问题（仅首轮前展示） -->
        <div
          v-if="rounds.length === 0"
          class="flex flex-col items-center gap-3 py-10 text-center"
        >
          <span class="text-5xl text-blue-500">⚡</span>
          <div class="text-lg font-semibold">{{ $t('page.aiQuery.title') }}</div>
          <div class="text-sm text-gray-400">{{ $t('page.aiQuery.emptyDesc') }}</div>
          <div class="mt-2 flex flex-wrap justify-center gap-2">
            <a-button
              v-for="key in sampleKeys"
              :key="key"
              size="small"
              @click="input = sampleText(key)"
            >
              {{ sampleText(key) }}
            </a-button>
          </div>
        </div>

        <div class="flex flex-col gap-3">
          <div v-for="(round, i) in rounds" :key="i" class="flex flex-col gap-3">
            <!-- 用户问题：右侧气泡 -->
            <div class="flex flex-row-reverse items-start gap-3">
              <div
                class="max-w-[80%] whitespace-pre-wrap break-words rounded-2xl rounded-tr-sm bg-blue-500 px-4 py-2 text-white"
              >
                {{ round.question }}
              </div>
            </div>

            <!-- 结果卡片：左侧 -->
            <div class="flex items-start gap-3">
              <div
                class="min-w-0 flex-1 rounded-xl border border-solid border-gray-200 bg-card p-4 dark:border-gray-700"
              >
                <div
                  v-if="round.loading"
                  class="flex items-center gap-2 text-sm text-gray-400"
                >
                  <span class="inline-block h-2 w-2 animate-pulse rounded-full bg-blue-500" />
                  {{ $t('page.aiQuery.thinking') }}
                </div>
                <template v-else>
                  <details class="mb-3" open>
                    <summary class="cursor-pointer text-xs text-gray-400">
                      {{ $t('page.aiQuery.generatedSql') }}
                    </summary>
                    <pre
                      class="mt-2 overflow-x-auto rounded-lg bg-gray-900 p-3 text-xs leading-relaxed text-gray-100"
                      >{{ round.sql }}</pre
                    >
                  </details>

                  <a-table
                    v-if="round.columns && round.columns.length > 0"
                    :columns="round.columns.map((c, i) => ({ title: c, dataIndex: String(i) }))"
                    :data-source="(round.rows || []).map((r, ri) => ({ key: ri, cells: r.cells }))"
                    :pagination="
                      (round.rows?.length || 0) > 10
                        ? { pageSize: 10, showSizeChanger: false }
                        : false
                    "
                    size="small"
                  >
                    <template #bodyCell="{ column, record }">
                      {{ record.cells[Number(column.dataIndex)] }}
                    </template>
                  </a-table>
                  <div
                    v-if="round.sql && (round.rows?.length ?? 0) === 0"
                    class="mt-2 text-xs text-gray-400"
                  >
                    {{ $t('page.aiQuery.noRows') }}
                  </div>

                  <div
                    v-if="round.answer"
                    class="mt-3 rounded-lg border border-solid border-blue-900 bg-blue-950 p-3 text-sm leading-relaxed"
                  >
                    <a-tag color="processing">{{ $t('page.aiQuery.answerTag') }}</a-tag>
                    <div class="markdown-body break-words" v-html="md(round.answer)"></div>
                  </div>
                </template>
              </div>
            </div>
          </div>
        </div>
        <div ref="bottomRef"></div>
      </div>

      <!-- 输入区（吸底） -->
      <div
        class="shrink-0 rounded-xl border border-solid border-gray-200 bg-card p-3 dark:border-gray-700"
      >
        <div class="flex items-end gap-2">
          <a-textarea
            v-model:value="input"
            :auto-size="{ minRows: 1, maxRows: 4 }"
            :disabled="loading"
            :placeholder="$t('page.aiQuery.inputPlaceholder')"
            @keydown.enter.exact.prevent="handleAsk()"
          />
          <a-button
            :disabled="!input.trim()"
            :loading="loading"
            type="primary"
            @click="handleAsk()"
          >
            {{ $t('page.aiQuery.ask') }}
          </a-button>
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

.markdown-body :deep(table) {
  border-collapse: collapse;
  margin: 0.5em 0;
  width: 100%;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid var(--border);
  padding: 4px 8px;
}

.markdown-body :deep(th) {
  background: rgb(128 128 128 / 10%);
}
</style>
