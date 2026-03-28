<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useColorMode } from '@vueuse/core'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'

import { useSettingsStore } from '@/stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const mode = useColorMode()

function handleThemeChange(theme: 'light' | 'dark' | 'auto') {
  mode.value = theme
  settingsStore.setSetting<string>('theme', theme)
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button
        v-motion
        :hovered="{ scale: 1.05 }"
        :tapped="{ scale: 0.95 }"
        variant="outline"
      >
        <Icon icon="hugeicons:sun-01" class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all duration-300 dark:-rotate-90 dark:scale-0" />
        <Icon icon="hugeicons:moon-02" class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all duration-300 dark:rotate-0 dark:scale-100" />
        <span class="sr-only">{{ t('settings.theme.label') }}</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end">
      <DropdownMenuItem
        v-motion
        :initial="{ opacity: 0, x: -10 }"
        :enter="{ opacity: 1, x: 0, transition: { delay: 0 } }"
        :hovered="{ x: 4 }"
        @click="handleThemeChange('light')"
      >
        {{ t('settings.theme.light') }}
      </DropdownMenuItem>
      <DropdownMenuItem
        v-motion
        :initial="{ opacity: 0, x: -10 }"
        :enter="{ opacity: 1, x: 0, transition: { delay: 50 } }"
        :hovered="{ x: 4 }"
        @click="handleThemeChange('dark')"
      >
        {{ t('settings.theme.dark') }}
      </DropdownMenuItem>
      <DropdownMenuItem
        v-motion
        :initial="{ opacity: 0, x: -10 }"
        :enter="{ opacity: 1, x: 0, transition: { delay: 100 } }"
        :hovered="{ x: 4 }"
        @click="handleThemeChange('auto')"
      >
        {{ t('settings.theme.system') }}
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
