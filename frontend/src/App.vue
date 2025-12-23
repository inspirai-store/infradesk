<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, darkTheme } from 'naive-ui'
import { computed } from 'vue'
import AppLayout from './components/AppLayout.vue'

const themeOverrides = computed(() => ({
  common: {
    primaryColor: '#00FFFF',
    primaryColorHover: '#33FFFF',
    primaryColorPressed: '#00CCCC',
    primaryColorSuppl: '#FF00FF',
    borderRadius: '8px',
    fontFamily: "'JetBrains Mono', monospace",
  },
  Button: {
    textColorPrimary: '#0a0a1a',
  },
  Card: {
    borderRadius: '12px',
  },
  DataTable: {
    borderRadius: '8px',
  },
}))
</script>

<template>
  <NConfigProvider :theme="darkTheme" :theme-overrides="themeOverrides">
    <NMessageProvider>
      <NDialogProvider>
        <AppLayout />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
#app {
  width: 100%;
  height: 100vh;
}
</style>

