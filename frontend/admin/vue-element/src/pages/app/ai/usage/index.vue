<template>
  <div class="app-container h-full flex flex-1 flex-col">
    <!-- 汇总卡 -->
    <el-row :gutter="16" class="mb-4">
      <el-col v-for="m in metricItems" :key="m.label" :xs="24" :sm="8">
        <el-card shadow="hover">
          <div class="metric-label">{{ m.label }}</div>
          <div class="metric-value" :style="{ color: m.color }">{{ m.value }}</div>
          <el-progress
            v-if="m.progress !== undefined"
            :percentage="m.progress"
            :status="m.progress > 80 ? 'exception' : 'success'"
          />
        </el-card>
      </el-col>
    </el-row>

    <!-- 流水列表 -->
    <el-card shadow="hover" class="flex-1">
      <el-table :data="usageRows" size="small" v-loading="loading" border stripe>
        <el-table-column prop="modelName" :label="t('pages.ai_usage.model')" min-width="160" show-overflow-tooltip />
        <el-table-column prop="promptTokens" :label="t('pages.ai_usage.promptTokens')" width="120" />
        <el-table-column prop="completionTokens" :label="t('pages.ai_usage.completionTokens')" width="130" />
        <el-table-column prop="totalTokens" :label="t('pages.ai_usage.totalTokens')" width="110" />
        <el-table-column :label="t('pages.ai_usage.duration')" width="100">
          <template #default="{ row }">{{ row.durationMs || 0 }} ms</template>
        </el-table-column>
        <el-table-column prop="createdAt" :label="t('pages.ai_usage.time')" width="180" />
      </el-table>
      <div class="mt-3 flex justify-end">
        <el-pagination
          v-model:current-page="page"
          :page-size="20"
          layout="total, prev, pager, next"
          :total="total"
          @current-change="loadRows"
        />
      </div>
    </el-card>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref } from "vue";
import { ElCard, ElCol, ElProgress, ElRow, ElTable, ElTableColumn, ElPagination } from "element-plus";
import { useI18n } from "@/core/i18n";
import { PaginationQuery } from "@/core/transport/rest";
import { apiClient } from "@/api/client";

const { t } = useI18n();

const monthTokens = ref(0);
const monthCalls = ref(0);
const quotaConfigured = ref(false);
const quotaLimit = ref(0);

const usageRows = ref<Record<string, any>[]>([]);
const total = ref(0);
const page = ref(1);
const loading = ref(false);

const metricItems = computed(() => {
  const pct = quotaConfigured.value && quotaLimit.value > 0
    ? Math.min(100, Math.round((monthTokens.value * 100) / quotaLimit.value))
    : undefined;
  return [
    { label: t("pages.ai_usage.monthTokens"), value: monthTokens.value.toLocaleString(), color: "#3b82f6", progress: pct },
    { label: t("pages.ai_usage.monthCalls"), value: monthCalls.value.toLocaleString(), color: "#22d3ee", progress: undefined },
    { label: t("pages.ai_usage.quota"), value: quotaConfigured.value ? quotaLimit.value.toLocaleString() : "∞", color: "#a78bfa", progress: undefined },
  ];
});

async function loadSummary() {
  try {
    const resp = await apiClient.aiUsageLogService.GetUsageSummary({});
    monthTokens.value = resp.monthTokens ?? 0;
    monthCalls.value = resp.monthCalls ?? 0;
    quotaConfigured.value = resp.quotaConfigured ?? false;
    quotaLimit.value = resp.quotaLimit ?? 0;
  } catch (error: any) {
    console.error("load usage summary failed:", error);
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
  } catch (error: any) {
    console.error("load usage rows failed:", error);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadSummary();
  loadRows();
});
</script>

<style lang="scss" scoped>
.metric-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 6px;
}

.metric-value {
  font-size: 24px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
</style>
