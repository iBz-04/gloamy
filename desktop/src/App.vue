<script setup lang="ts">
import { ConfigProvider } from 'reka-ui'
import AppSidebar from '@/components/AppSidebar.vue'
import { TooltipProvider } from '@/components/ui/tooltip'
</script>

<template>
  <ConfigProvider>
    <TooltipProvider>
      <!-- Removed top-right square for macOS window controls compatibility -->

      <div class="flex h-screen w-full bg-background text-foreground font-sans">
        <AppSidebar />
        <main class="flex-1 overflow-hidden flex flex-col">
          <router-view v-slot="{ Component }">
            <Transition name="page" mode="out-in">
              <component :is="Component" />
            </Transition>
          </router-view>
        </main>
      </div>
    </TooltipProvider>
  </ConfigProvider>
</template>

<style>
.page-enter-active,
.page-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
