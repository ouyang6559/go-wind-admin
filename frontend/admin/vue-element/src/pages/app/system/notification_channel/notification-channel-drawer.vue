<template>
  <ProModal
    v-model:visible="drawer.visible.value"
    :title="drawer.title.value"
    :loading="drawer.pageLoading.value"
    :config="{
      component: 'drawer',
      drawer: { size: drawer.drawerWidth, closeOnClickModal: false },
    }"
  >
    <ElForm
      ref="formRef"
      :model="drawer.formData"
      :rules="formRules"
      label-width="140px"
      class="drawer-form"
    >
      <ElFormItem :label="t('pages.notification_channel.name')" prop="name">
        <ElInput v-model="drawer.formData.name" :placeholder="t('pages.notification_channel.requiredName')" clearable />
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.type')" prop="type">
        <ElSelect v-model="drawer.formData.type" :disabled="!isCreate">
          <ElOption :label="t('pages.notification_channel.typeEmail')" value="EMAIL" />
        </ElSelect>
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.smtpHost')" prop="smtpHost">
        <ElInput v-model="drawer.formData.smtpHost" placeholder="smtp.example.com" clearable />
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.smtpPort')" prop="smtpPort">
        <ElInputNumber
          v-model="drawer.formData.smtpPort"
          :min="1"
          :max="65535"
          :precision="0"
          controls-position="right"
        />
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.smtpUsername')" prop="smtpUsername">
        <ElInput v-model="drawer.formData.smtpUsername" clearable />
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.password')" prop="password">
        <ElInput
          v-model="drawer.formData.password"
          type="password"
          show-password
          :placeholder="isCreate ? t('pages.notification_channel.passwordPlaceholder') : t('pages.notification_channel.passwordKeepHint')"
        />
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.smtpFrom')" prop="smtpFrom">
        <ElInput v-model="drawer.formData.smtpFrom" placeholder="noreply@example.com" clearable />
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.smtpTls')" prop="smtpTls">
        <ElSelect v-model="drawer.formData.smtpTls">
          <ElOption :label="t('pages.notification_channel.tlsNone')" value="NONE" />
          <ElOption :label="t('pages.notification_channel.tlsStartTls')" value="START_TLS" />
          <ElOption :label="t('pages.notification_channel.tlsSsl')" value="SSL" />
        </ElSelect>
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.enabled')" prop="enabled">
        <ElSwitch v-model="drawer.formData.enabled" />
      </ElFormItem>

      <ElFormItem :label="t('pages.notification_channel.remark')" prop="remark">
        <ElInput v-model="drawer.formData.remark" type="textarea" :rows="2" />
      </ElFormItem>
    </ElForm>

    <template #footer>
      <div class="drawer-footer">
        <ElButton @click="drawer.close">{{ $t("common.button.cancel") }}</ElButton>
        <ElButton
          type="primary"
          :loading="drawer.submitLoading.value"
          @click="drawer.handleSubmit(formRef, () => emit('success'))"
        >
          {{ $t("common.button.confirm") }}
        </ElButton>
      </div>
    </template>
  </ProModal>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import {
  ElButton,
  ElForm,
  ElFormItem,
  ElInput,
  ElInputNumber,
  ElOption,
  ElSelect,
  ElSwitch,
} from "element-plus";

import ProModal from "@/components/Pro/ProModal/index.vue";
import { useDrawerForm } from "@/components/Pro/composables/useDrawerForm";
import { useI18n } from "@/core/i18n";
import {
  createNotificationChannel,
  updateNotificationChannel,
} from "@/api/composables";

const emit = defineEmits<{
  success: [];
}>();

const { t } = useI18n();

const formRef = ref();
const isCreate = ref(true);

const drawer = useDrawerForm({
  moduleKey: "pages.notification_channel.moduleName",
  defaults: {
    name: "",
    type: "EMAIL",
    smtpHost: "",
    smtpPort: 587,
    smtpUsername: "",
    password: "",
    smtpFrom: "",
    smtpTls: "START_TLS",
    enabled: true,
    remark: "",
  },
  createFn: async (values: Record<string, any>) => {
    const { password, ...data } = values;
    return createNotificationChannel(data, password);
  },
  updateFn: (id: number, values: Record<string, any>) => {
    const { password, ...data } = values;
    return updateNotificationChannel(id, data, password || undefined);
  },
});

const formRules = computed(() => ({
  name: [{ required: true, message: t("pages.notification_channel.requiredName"), trigger: "blur" }],
  password: isCreate.value
    ? [{ required: true, message: t("pages.notification_channel.requiredPassword"), trigger: "blur" }]
    : [],
}));

// 包装 open：追踪创建/编辑模式（切换类型字段禁用态与密码必填规则），
// 并在编辑时显式回填行数据（useDrawerForm 不做默认填充；hasPassword 为
// 服务端计算字段、密码不回显，均不进表单）。
function open(options: { create: boolean; row?: any }) {
  isCreate.value = options.create;
  drawer.open(options, (row: any) => {
    Object.assign(drawer.formData, {
      name: row.name || "",
      type: row.type || "EMAIL",
      smtpHost: row.smtpHost || "",
      smtpPort: row.smtpPort ?? 587,
      smtpUsername: row.smtpUsername || "",
      smtpFrom: row.smtpFrom || "",
      smtpTls: row.smtpTls || "START_TLS",
      enabled: !!row.enabled,
      remark: row.remark || "",
    });
  });
}

defineExpose({ open });
</script>

<style lang="scss" scoped>
.drawer-form {
  padding-right: 10px;
}

.drawer-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
