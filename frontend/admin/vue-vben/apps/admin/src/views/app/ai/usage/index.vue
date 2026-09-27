<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { $t } from '@vben/locales';

import { apiClient, PaginationQuery } from '#/api';

const monthTokens = ref(0);
const monthCalls = ref(0);
const quotaConfigured = ref(false);
const quotaLimit = ref(0);

const usageRows = ref<Record<string, any>[]>([]);
const total = ref(0);
const page = ref(1);
const loading = ref(false);

const metricItems = computed(() => [
  {
    color: '#3b82f6',
    label: $t('page.aiUsage.monthTokens'),
    value: monthTokens.value,
  },
  {
    color: '#22d3ee',
    label: $t('page.aiUsage.monthCalls'),
    value: monthCalls.value,
  },
  {
    color: '#a78bfa',
    label: $t('page.aiUsage.quota'),
    value: quotaConfigured.value ? `${quotaLimit.value}` : '∞',
  },
]);

async function loadSummary() {
  try {
    const resp = await apiClient.aiUsageLogService.GetUsageSummary({});
    monthTokens.value = resp.monthTokens ?? 0;
    monthCalls.value = resp.monthCalls ?? 0;
    quotaConfigured.value = resp.quotaConfigured ?? false;
    quotaLimit.value = resp.quotaLimit ?? 0;
  } catch (error) {
    console.error('load usage summary failed:', error);
  }
}

async function loadRows() {
  loading.value = true;
  try {
    const resp = await apiClient.aiUsageLogService.List(
      new PaginationQuery({ paging: { page: page.value, pageSize: 20 } }).toRawParams(),
    );
    usageRows.value = (resp.items || []) as Record<string, any>[];
    total.value = Number(resp.total || 0);
  } catch (error) {
    console.error('load usage rows failed:', error);
  } finally {
    loading.value = false;
  }
}

const columns = [
  { title: $t('page.aiUsage.model'), dataIndex: 'modelName' },
  { title: $t('page.aiUsage.promptTokens'), dataIndex: 'promptTokens', width: 120 },
  { title: $t('page.aiUsage.completionTokens'), dataIndex: 'completionTokens', width: 130 },
  { title: $t('page.aiUsage.totalTokens'), dataIndex: 'totalTokens', width: 110 },
  { title: $t('page.aiUsage.duration'), dataIndex: 'durationMs', width: 100 },
  { title: $t('page.aiUsage.time'), dataIndex: 'createdAt', width: 180 },
];

onMounted(() => {
  loadSummary();
  loadRows();
});
</script>

<template>
  <Page auto-content-height>
    <div class="flex h-full flex-col gap-4">
      <!-- 汇总卡 -->
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <div
          v-for="m in metricItems"
          :key="m.label"
          class="rounded-xl border border-solid border-gray-200 bg-card p-4 dark:border-gray-700"
        >
          <div class="mb-2 text-sm text-gray-400">{{ m.label }}</div>
          <div class="text-2xl font-semibold tabular-nums" :style="{ color: m.color }">
            {{ m.value }}
          </div>
        </div>
      </div>

      <!-- 流水列表 -->
      <div
        class="min-h-0 flex-1 rounded-xl border border-solid border-gray-200 bg-card p-4 dark:border-gray-700"
      >
        <a-table
          :columns="columns"
          :data-source="usageRows"
          :loading="loading"
          :pagination="{ pageSize: 20, showSizeChanger: false }"
          :scroll="{ y: 'calc(100% - 40px)' }"
          row-key="id"
          size="small"
        />
      </div>
    </div>
  </Page>
</template>
