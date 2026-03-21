<script setup lang="ts">
import { ConfigProvider } from 'reka-ui'
import AppSidebar from '@/components/AppSidebar.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import { TooltipProvider } from '@/components/ui/tooltip'
</script>

<template>
  <ConfigProvider>
    <TooltipProvider>
      <div class="flex h-screen w-full bg-background text-foreground font-sans">
        <AppSidebar />
        <main class="flex-1 overflow-hidden flex flex-col">
          <AppTopbar />
          <div class="flex-1 overflow-hidden">
            <router-view v-slot="{ Component }">
              <Transition name="page" mode="out-in">
                <component :is="Component" />
              </Transition>
            </router-view>
          </div>
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
