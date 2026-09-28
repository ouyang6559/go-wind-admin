import type { EChartsOption } from "echarts";

import type EchartsUI from "./echarts-ui.vue";

import type { Ref } from "vue";
import { computed, nextTick, watch } from "vue";

import { usePreferences } from "@/core/preferences";

import {
  tryOnUnmounted,
  useDebounceFn,
  useResizeObserver,
  useTimeoutFn,
  useWindowSize,
} from "@vueuse/core";

import echarts from "./echarts";

type EchartsUIType = typeof EchartsUI | undefined;

type EchartsThemeType = "dark" | "light" | null;

function useEcharts(chartRef: Ref<EchartsUIType>) {
  let chartInstance: echarts.ECharts | null = null;
  let cacheOptions: EChartsOption = {};

  const { isDark } = usePreferences();
  const { height, width } = useWindowSize();
  const resizeHandler: () => void = useDebounceFn(resize, 200);

  const getOptions = computed((): EChartsOption => {
    if (!isDark.value) {
      return {};
    }

    return {
      backgroundColor: "transparent",
    };
  });

  const initCharts = (t?: EchartsThemeType) => {
    const el = chartRef?.value?.$el;
    if (!el) {
      return;
    }
    chartInstance = echarts.init(el, t || isDark.value ? "dark" : null);

    return chartInstance;
  };

  const renderEcharts = (options: EChartsOption, clear = true) => {
    cacheOptions = options;
    const currentOptions = {
      ...options,
      ...getOptions.value,
    };
    return new Promise((resolve) => {
      if (chartRef.value?.offsetHeight === 0) {
        useTimeoutFn(() => {
          renderEcharts(currentOptions);
          resolve(null);
        }, 30);
        return;
      }
      nextTick(() => {
        useTimeoutFn(() => {
          const el = chartRef?.value?.$el;
          if (!el) {
            return;
          }
          // v-if 切走再切回（图表/表格视图切换、折叠展开）会重挂载出全新 DOM，
          // 旧实例仍附着在已脱离文档的节点上，setOption 画得再对也不可见——必须重建
          if (chartInstance && chartInstance.getDom() !== el) {
            chartInstance.dispose();
            chartInstance = null;
          }
          if (!chartInstance) {
            chartInstance = echarts.init(el, isDark.value ? "dark" : null);
          }
          if (clear) {
            chartInstance?.clear();
          }
          chartInstance?.setOption(currentOptions);
          resolve(null);
        }, 30);
      });
    });
  };

  function resize() {
    chartInstance?.resize({
      animation: {
        duration: 300,
        easing: "quadraticIn",
      },
    });
  }

  watch([width, height], () => {
    resizeHandler?.();
  });

  useResizeObserver(chartRef as never, resizeHandler);

  watch(isDark, () => {
    if (chartInstance) {
      chartInstance.dispose();
      initCharts();
      renderEcharts(cacheOptions);
      resize();
    }
  });

  tryOnUnmounted(() => {
    // 销毁实例，释放资源
    chartInstance?.dispose();
  });
  return {
    renderEcharts,
    resize,
  };
}

export { useEcharts };

export type { EchartsUIType };
