<template>
  <div class="app-container ai-query-page h-full flex flex-1 flex-col">
    <div class="mx-auto flex w-full max-w-4xl flex-1 flex-col gap-4 overflow-y-auto pb-2">
      <!-- 标题 + 示例问题（仅首轮前展示） -->
      <div v-if="rounds.length === 0" class="flex flex-col items-center gap-3 py-10 text-center">
        <Icon icon="lucide:zap" class="text-5xl text-blue-500" />
        <div class="text-lg font-semibold">{{ t("pages.ai_query.title") }}</div>
        <div class="text-sm text-gray-400">{{ t("pages.ai_query.emptyDesc") }}</div>
        <div class="mt-2 flex flex-wrap justify-center gap-2">
          <ElButton
            v-for="key in sampleKeys"
            :key="key"
            size="small"
            @click="input = t(`pages.ai_query.${key}`)"
          >
            {{ t(`pages.ai_query.${key}`) }}
          </ElButton>
        </div>
      </div>

      <!-- 问答轮次 -->
      <div v-for="(round, i) in rounds" :key="i" class="flex flex-col gap-3">
        <!-- 用户问题：右侧气泡 -->
        <div class="flex flex-row-reverse items-start gap-3">
          <div class="max-w-[80%] whitespace-pre-wrap break-words rounded-2xl rounded-tr-sm bg-blue-500 px-4 py-2 text-white">
            {{ round.question }}
          </div>
        </div>

        <!-- 结果卡片：左侧 -->
        <div class="flex items-start gap-3">
          <div class="min-w-0 flex-1 rounded-xl border border-solid border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-900">
            <div v-if="round.loading" class="flex items-center gap-2 text-sm text-gray-400">
              <span class="inline-block h-2 w-2 animate-pulse rounded-full bg-blue-500" />
              {{ t("pages.ai_query.thinking") }}
            </div>
            <template v-else>
              <details v-if="round.sql" class="mb-3" open>
                <summary class="cursor-pointer text-xs text-gray-400">
                  {{ t("pages.ai_query.generatedSql") }}
                </summary>
                <pre class="mt-2 overflow-x-auto rounded-lg bg-gray-900 p-3 text-xs leading-relaxed text-gray-100">{{ round.sql }}</pre>
              </details>

              <ElTable v-if="round.columns && round.columns.length > 0" :data="tableRows(round)" size="small" border>
                <ElTableColumn
                  v-for="(col, ci) in round.columns"
                  :key="col"
                  :label="col"
                  :prop="String(ci)"
                  show-overflow-tooltip
                >
                  <template #default="{ row }">{{ row.cells[ci] }}</template>
                </ElTableColumn>
              </ElTable>
              <div v-if="round.sql && (round.rows?.length ?? 0) === 0" class="mt-2 text-xs text-gray-400">
                {{ t("pages.ai_query.noRows") }}
              </div>

              <div
                v-if="round.answer"
                class="mt-3 rounded-lg border border-solid border-blue-200 bg-blue-50 p-3 text-sm leading-relaxed dark:border-blue-900 dark:bg-blue-950"
              >
                <ElTag type="primary" size="small" class="mb-1">{{ t("pages.ai_query.answerTag") }}</ElTag>
                <div class="markdown-body break-words" v-html="renderMarkdown(round.answer)" />
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入区（吸底） -->
    <div class="sticky bottom-0 mt-4 border-t border-solid border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-900">
      <div class="mx-auto flex w-full max-w-4xl items-end gap-2">
        <ElInput
          v-model="input"
          type="textarea"
          :autosize="{ minRows: 1, maxRows: 4 }"
          :placeholder="t('pages.ai_query.inputPlaceholder')"
          :disabled="loading"
          @keydown.enter.exact.prevent="handleAsk()"
        />
        <ElButton type="primary" :loading="loading" :disabled="!input.trim()" @click="handleAsk()">
          {{ loading ? t("pages.ai_query.thinking") : t("pages.ai_query.ask") }}
        </ElButton>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { nextTick, ref } from "vue";
import { ElButton, ElInput, ElMessage, ElTable, ElTableColumn, ElTag } from "element-plus";
import { Icon } from "@iconify/vue";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { i18n, useI18n } from "@/core/i18n";
import { apiClient } from "@/api/client";

// marked 单行换行按 GFM 处理
marked.setOptions({ gfm: true, breaks: true });

function renderMarkdown(content: string): string {
  // AI 回复经 DOMPurify 消毒后再 v-html；LLM 输出不可信，防注入
  return DOMPurify.sanitize(marked.parse(content, { async: false }) as string);
}

const { t } = useI18n();

const sampleKeys = ["sample1", "sample2", "sample3"];

interface Round {
  question: string;
  sql?: string;
  columns?: string[];
  rows?: { cells: string[] }[];
  answer?: string;
  loading: boolean;
}

const rounds = ref<Round[]>([]);
const input = ref("");
const loading = ref(false);

function tableRows(round: Round) {
  return (round.rows || []).map((r, i) => ({ id: i, cells: r.cells }));
}

/** 滚到底部（新轮次追加后）。 */
function scrollToBottom() {
  nextTick(() => {
    window.scrollTo({ top: document.body.scrollHeight, behavior: "smooth" });
  });
}

/** 告警文案：后端只回结构化事实，LLM 结论走 markdown 渲染，无需模板。 */
async function handleAsk() {
  const question = input.value.trim();
  if (!question || loading.value) return;
  // 携带最近 5 轮历史（问题+SQL+结果摘要），供模型消解追问里的指代
  const history = rounds.value
    .filter((r) => !r.loading && r.sql)
    .slice(-5)
    .map((r) => ({
      question: r.question,
      sql: r.sql || "",
      resultSummary: (r.rows || []).slice(0, 3).map((row) => row.cells.join(" | ")).join("；"),
    }));
  rounds.value.push({ question, loading: true });
  input.value = "";
  loading.value = true;
  scrollToBottom();
  try {
    const locale = i18n.global.locale as unknown as { value?: string };
    const lang = locale?.value || "zh-CN";
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
      last.rows = (resp.rows || []).map((r) => ({ cells: (r.values || []) as string[] }));
      last.answer = resp.answer || "";
      last.loading = false;
    }
    scrollToBottom();
  } catch (error: any) {
    console.error("ai query failed:", error);
    ElMessage.error(error?.message || t("pages.ai_query.failed"));
    const last = rounds.value[rounds.value.length - 1];
    if (last) last.loading = false;
  }
}
</script>

<style lang="scss" scoped>
.ai-query-page {
  min-height: 0;
}

.markdown-body {
  :deep(p) {
    margin: 0 0 0.5em;
    &:last-child {
      margin-bottom: 0;
    }
  }
  :deep(table) {
    border-collapse: collapse;
    margin: 0.5em 0;
    width: 100%;
  }
  :deep(th),
  :deep(td) {
    border: 1px solid var(--el-border-color);
    padding: 4px 8px;
  }
  :deep(th) {
    background: var(--el-fill-color-light);
  }
}
</style>
