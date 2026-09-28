<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { $t } from '@vben/locales';
import { dateUtil } from '@vben/utils';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { apiClient, PaginationQuery } from '#/api';

const monthTokens = ref(0);
const monthCalls = ref(0);
const quotaConfigured = ref(false);
const quotaLimit = ref(0);

const metricItems = computed(() => [
  {
    color: '#3b82f6',
    label: $t('page.aiUsage.monthTokens'),
    value: monthTokens.value.toLocaleString(),
  },
  {
    color: '#22d3ee',
    label: $t('page.aiUsage.monthCalls'),
    value: monthCalls.value.toLocaleString(),
  },
  {
    color: '#a78bfa',
    label: $t('page.aiUsage.quota'),
    value: quotaConfigured.value ? quotaLimit.value.toLocaleString() : '∞',
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

/** 流水列表 fetcher：List + 分页参数（Grid proxy 复用） */
async function fetchListAiUsageLogs(query: PaginationQuery) {
  return apiClient.aiUsageLogService.List(query.toRawParams());
}

const formOptions = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'modelName',
      label: $t('page.aiUsage.model'),
      componentProps: { allowClear: true },
    },
  ],
};

const gridOptions: VxeGridProps<Record<string, any>> = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  rowConfig: { isHover: true, keyField: 'id' },
  stripe: true,
  proxyConfig: {
    ajax: {
      query: async ({ page }, formValues) => {
        return await fetchListAiUsageLogs(
          new PaginationQuery({
            paging: { page: page.currentPage, pageSize: page.pageSize },
            // formValues 字符串值由 PaginationQuery 统一转 __contains（搜索铁律）
            formValues:
              formValues && Object.keys(formValues).length > 0
                ? { ...formValues }
                : undefined,
          }),
        );
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 50 },
    { title: $t('page.aiUsage.model'), field: 'modelName', minWidth: 180 },
    {
      title: $t('page.aiUsage.promptTokens'),
      field: 'promptTokens',
      width: 130,
      formatter: ({ cellValue }) => Number(cellValue ?? 0).toLocaleString(),
    },
    {
      title: $t('page.aiUsage.completionTokens'),
      field: 'completionTokens',
      width: 140,
      formatter: ({ cellValue }) => Number(cellValue ?? 0).toLocaleString(),
    },
    {
      title: $t('page.aiUsage.totalTokens'),
      field: 'totalTokens',
      width: 110,
      formatter: ({ cellValue }) => Number(cellValue ?? 0).toLocaleString(),
    },
    {
      title: $t('page.aiUsage.duration'),
      field: 'durationMs',
      width: 110,
      formatter: ({ cellValue }) => `${Number(cellValue ?? 0)} ms`,
    },
    {
      title: $t('page.aiUsage.time'),
      field: 'createdAt',
      width: 180,
      formatter: ({ cellValue }) =>
        cellValue ? dateUtil(cellValue).format('YYYY-MM-DD HH:mm:ss') : '',
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions, formOptions });

onMounted(() => {
  loadSummary();
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

      <!-- 流水列表：vxe Grid 统一搜索/服务端分页约定（与其余管理页一致） -->
      <div class="min-h-0 flex-1">
        <Grid />
      </div>
    </div>
  </Page>
</template>
