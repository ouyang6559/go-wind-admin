<template>
  <ProPage ref="pageRef" :config="pageConfig" @add="handleAdd" @edit="handleEdit">
    <!-- 状态 -->
    <template #status="scope: any">
      <ElTag size="small" :type="scope.row.status === 'ON' ? 'success' : 'info'">
        {{ scope.row.status === "ON" ? t("pages.access_key.statusMap.ON") : t("pages.access_key.statusMap.OFF") }}
      </ElTag>
    </template>
  </ProPage>

  <!-- 创建/编辑抽屉 -->
  <AccessKeyDrawer ref="drawerRef" @success="handleSuccess" />
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { ElTag } from "element-plus";

import ProPage from "@/components/Pro/ProPage/index.vue";
import type { ProPageConfig } from "@/components/Pro/ProPage/types";
import AccessKeyDrawer from "./access-key-drawer.vue";
import { useI18n } from "@/core/i18n";
import {
  createPagedExportAction,
  deleteAccessKey,
  fetchListAccessKeys,
} from "@/api/composables";
import { PaginationQuery } from "@/core/transport/rest";

const { t } = useI18n();

const pageRef = ref();
const drawerRef = ref();

const pageConfig = computed<ProPageConfig>(() => ({
  skeleton: true,
  search: {
    grid: true,
    fields: [
      {
        type: "input",
        label: t("pages.access_key.name"),
        field: "name",
        attrs: { placeholder: t("common.placeholder.input"), clearable: true },
      },
    ],
  },

  table: {
    listAction: async (query: any) => {
      const { page, pageSize, ...queryParams } = query;
      const result = await fetchListAccessKeys(
        new PaginationQuery({
          paging: { page: page || 1, pageSize: pageSize || 10 },
          formValues: queryParams,
        }),
      );
      return { items: result.items || [], total: result.total || 0 };
    },
    deleteAction: async (ids: string) => {
      await deleteAccessKey(ids);
    },
    exportsAction: createPagedExportAction(fetchListAccessKeys),
    toolbar: [],
    toolbarRight: ["add"],
    defaultToolbar: ["refresh", "exports", "filter"],
    tableAttrs: { border: true, stripe: true },
    emptyActionText: "common.button.add",
    columns: [
      { type: "index", label: t("common.table.seq"), width: 60 },
      { prop: "name", label: t("pages.access_key.name"), minWidth: 160 },
      { prop: "accessKey", label: t("pages.access_key.accessKey"), minWidth: 220 },
      {
        prop: "status",
        label: t("pages.access_key.status"),
        minWidth: 90,
        slotName: "status",
      },
      {
        prop: "expiresAt",
        label: t("pages.access_key.expiresAt"),
        minWidth: 160,
        cellType: "date",
        dateFormat: "YYYY-MM-DD HH:mm:ss",
      },
      {
        prop: "lastUsedAt",
        label: t("pages.access_key.lastUsedAt"),
        minWidth: 160,
        cellType: "date",
        dateFormat: "YYYY-MM-DD HH:mm:ss",
      },
      {
        prop: "createdAt",
        label: t("common.table.createdAt"),
        minWidth: 160,
        cellType: "date",
        dateFormat: "YYYY-MM-DD HH:mm:ss",
      },
      {
        prop: "action",
        label: t("common.table.action"),
        fixed: "right",
        width: 150,
        cellType: "tool",
        buttons: [
          { name: "edit", label: t("common.button.edit"), icon: "lucide:pen-line" },
          {
            name: "delete",
            label: t("common.button.delete"),
            icon: "lucide:trash-2",
            attrs: { type: "danger" },
          },
        ],
      },
    ],
  },
}));

function handleAdd() {
  drawerRef.value?.open({ create: true });
}

function handleEdit(row: any) {
  drawerRef.value?.open({ create: false, row });
}

function handleSuccess() {
  pageRef.value?.refresh();
}
</script>

<style lang="scss" scoped>
:deep(.el-tag) {
  text-transform: none;
}
</style>
