<script setup lang="ts">
import { ref } from 'vue'
import { Icon } from '@iconify/vue'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

const selectedMode = ref('Work locally')
const currentBranch = ref('main')
const currentProject = ref('desktop')
</script>

<template>
  <div class="flex items-center gap-1.5 px-4 py-2 bg-black/[0.03] border border-black/[0.05] rounded-full w-fit mx-auto transition-all hover:bg-black/[0.04] hover:border-black/[0.08]">
    <!-- Project Selector -->
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <button class="flex items-center gap-1.5 px-2 py-1 text-[13px] font-medium text-black/60 hover:text-black transition-all rounded-md">
          <Icon icon="hugeicons:folder-01" class="size-4 opacity-50" />
          {{ currentProject }}
          <Icon icon="hugeicons:arrow-down-01-sharp" class="size-3 opacity-30" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" class="min-w-[160px] rounded-2xl p-1.5">
        <DropdownMenuItem @click="currentProject = 'desktop'" class="rounded-xl px-3 py-2 text-[13px]">desktop</DropdownMenuItem>
        <DropdownMenuItem @click="currentProject = 'core'" class="rounded-xl px-3 py-2 text-[13px]">core</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>

    <div class="w-px h-3 bg-black/10 mx-0.5" />

    <!-- Mode Selector -->
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <button class="flex items-center gap-1.5 px-2 py-1 text-[13px] font-medium text-black/60 hover:text-black transition-all rounded-md">
          <Icon icon="hugeicons:computer-01" class="size-4 opacity-50" />
          {{ selectedMode }}
          <Icon icon="hugeicons:arrow-down-01-sharp" class="size-3 opacity-30" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="center" class="min-w-[220px] rounded-2xl p-1.5 shadow-[0_10px_40px_rgba(0,0,0,0.08)] border-black/5">
        <div class="px-3 py-2 text-[11px] font-semibold text-black/30 uppercase tracking-wider">Continue in</div>
        <DropdownMenuItem @click="selectedMode = 'Work locally'" class="rounded-xl px-3 py-2.5 text-[13px] flex items-center justify-between group">
          <div class="flex items-center gap-2.5">
            <Icon icon="hugeicons:computer-01" class="size-4 opacity-40 group-hover:opacity-100" />
            Work locally
          </div>
          <Icon v-if="selectedMode === 'Work locally'" icon="hugeicons:tick-01" class="size-4" />
        </DropdownMenuItem>
        <DropdownMenuItem class="rounded-xl px-3 py-2.5 text-[13px] flex items-center gap-2.5 group">
          <Icon icon="hugeicons:git-branch-01" class="size-4 opacity-40 group-hover:opacity-100" />
          New worktree
        </DropdownMenuItem>
        <DropdownMenuItem class="rounded-xl px-3 py-2.5 text-[13px] flex items-center justify-between group">
          <div class="flex items-center gap-2.5">
            <Icon icon="hugeicons:globe-01" class="size-4 opacity-40 group-hover:opacity-100" />
            Connect gloamy web
          </div>
          <Icon icon="hugeicons:arrow-up-right-01" class="size-3.5 opacity-30" />
        </DropdownMenuItem>
        <DropdownMenuItem class="rounded-xl px-3 py-2.5 text-[13px] flex items-center gap-2.5 group opacity-40 grayscale">
          <Icon icon="hugeicons:cloud-01" class="size-4" />
          Send to cloud
        </DropdownMenuItem>
        <DropdownMenuSeparator class="my-1.5 bg-black/[0.04]" />
        <DropdownMenuItem class="rounded-xl px-3 py-2.5 text-[13px] flex items-center justify-between group">
          <div class="flex items-center gap-2.5 text-black/60">
            <Icon icon="hugeicons:clock-01" class="size-4 opacity-40 group-hover:opacity-100" />
            Rate limits remaining
          </div>
          <Icon icon="hugeicons:arrow-right-01-sharp" class="size-3.5 opacity-30" />
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>

    <div class="w-px h-3 bg-black/10 mx-0.5" />

    <!-- Branch Selector -->
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <button class="flex items-center gap-1.5 px-2 py-1 text-[13px] font-medium text-black/60 hover:text-black transition-all rounded-md">
          <Icon icon="hugeicons:git-branch-01" class="size-4 opacity-50" />
          {{ currentBranch }}
          <Icon icon="hugeicons:arrow-down-01-sharp" class="size-3 opacity-30" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" class="min-w-[160px] rounded-2xl p-1.5">
        <DropdownMenuItem class="rounded-xl px-3 py-2 text-[13px]">main</DropdownMenuItem>
        <DropdownMenuItem class="rounded-xl px-3 py-2 text-[13px]">develop</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>
