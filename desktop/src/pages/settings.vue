<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { useColorMode } from '@vueuse/core'
import { open } from '@tauri-apps/plugin-shell'
import { homeDir, join } from '@tauri-apps/api/path'

const auth = useAuthStore()
const settingsStore = useSettingsStore()
const mode = useColorMode()

const selectedTheme = ref<'light'|'dark'|'auto'>('auto')

onMounted(async () => {
  const t = await settingsStore.getSetting<string>('theme')
  if (t === 'light' || t === 'dark' || t === 'auto') {
    selectedTheme.value = t
  }
})

function handleThemeChange(theme: 'light' | 'dark' | 'auto') {
  selectedTheme.value = theme
  mode.value = theme
  settingsStore.setSetting<string>('theme', theme)
}

// language selector removed from settings page

const logout = async () => {
  await auth.logout()
}

async function openConfigFile() {
  const home = await homeDir()
  const configPath = await join(home, '.gloamy', 'config.toml')
  const ideTargets = [
    `vscode://file${configPath}`,
    `cursor://file${configPath}`,
  ]

  for (const target of ideTargets) {
    try {
      await open(target)
      return
    }
    catch {
    }
  }

  try {
    await open(configPath)
    return
  }
  catch {
  }

  await open(`file://${configPath}`)
}

function openWebsite() {
  open('https://www.gloamy.co')
}
</script>

<template>
  <div class="h-full flex flex-col overflow-y-auto bg-background text-[15px]">
    <!-- Header -->
    <div class="px-6 pt-6 pb-4">
      <h1 class="text-[16px] font-semibold text-foreground tracking-tight">Settings</h1>
    </div>

    <!-- Content -->
    <div class="px-6 pb-12 max-w-2xl space-y-8">

      <!-- Appearance Section -->
      <section class="space-y-4">
        <div class="flex items-center gap-2">
          <Icon icon="hugeicons:paint-board" class="size-[15px] text-muted-foreground" />
          <h2 class="text-[13px] font-medium text-muted-foreground">Appearance</h2>
        </div>

        <div class="flex flex-wrap gap-5">
          <!-- Light Mode Card -->
          <div class="flex flex-col items-center gap-2">
            <button
              @click="handleThemeChange('light')"
              class="relative w-[136px] h-[92px] rounded-xl bg-[#F8F9FA] border-[3px] transition-all p-1.5 focus:outline-none overflow-hidden"
              :class="selectedTheme === 'light' ? 'border-primary' : 'border-[#E2E8F0] hover:border-border'"
            >
              <div class="w-full h-full bg-white rounded-xl shadow-[0_1px_3px_rgba(0,0,0,0.05)] border border-[#E2E8F0] flex p-[5px] gap-1.5 pointer-events-none">
                <div class="w-[30%] h-full bg-[#F1F5F9] rounded border border-[#E2E8F0]"></div>
                <div class="flex-1 h-full bg-[#FFFFFF] rounded border border-[#E2E8F0] flex flex-col gap-1.5 p-1.5">
                  <div class="w-[60%] h-1.5 bg-[#CBD5E1] rounded-full"></div>
                  <div class="w-full h-1 bg-[#F1F5F9] rounded-full mt-1"></div>
                  <div class="w-[85%] h-1 bg-[#F1F5F9] rounded-full"></div>
                </div>
              </div>
            </button>
            <span class="text-[13px] font-medium text-muted-foreground" :class="{ 'text-foreground': selectedTheme === 'light' }">Light</span>
          </div>

          <!-- Dark Mode Card -->
          <div class="flex flex-col items-center gap-2">
            <button
              @click="handleThemeChange('dark')"
              class="relative w-[136px] h-[92px] rounded-xl bg-[#27272A] border-[3px] transition-all p-1.5 focus:outline-none overflow-hidden"
              :class="selectedTheme === 'dark' ? 'border-primary' : 'border-[#3F3F46] hover:border-[#52525B]'"
            >
              <div class="w-full h-full bg-[#09090B] rounded-xl shadow-[0_1px_4px_rgba(0,0,0,0.5)] border border-[#3F3F46] flex p-[5px] gap-1.5 pointer-events-none">
                <div class="w-[30%] h-full bg-[#18181B] rounded border border-[#3F3F46]"></div>
                <div class="flex-1 h-full bg-[#09090B] rounded border border-[#3F3F46] flex flex-col gap-1.5 p-1.5">
                  <div class="w-[60%] h-1.5 bg-[#52525B] rounded-full"></div>
                  <div class="w-full h-1 bg-[#27272A] rounded-full mt-1"></div>
                  <div class="w-[85%] h-1 bg-[#27272A] rounded-full"></div>
                </div>
              </div>
            </button>
            <span class="text-[13px] font-medium text-muted-foreground" :class="{ 'text-foreground': selectedTheme === 'dark' }">Dark</span>
          </div>

          <!-- System Mode Card -->
          <div class="flex flex-col items-center gap-2">
            <button
              @click="handleThemeChange('auto')"
              class="relative w-[136px] h-[92px] rounded-xl border-[3px] transition-all p-0 focus:outline-none overflow-hidden"
              :class="selectedTheme === 'auto' ? 'border-primary' : 'border-[#E2E8F0] dark:border-[#3F3F46] hover:border-border'"
            >
              <div class="absolute inset-0 flex rounded-xl overflow-hidden m-[3px]">
                <!-- Left (Light) -->
                <div class="w-1/2 h-full bg-[#F8F9FA] pt-1.5 pl-1.5 pb-1.5 pr-[2px]">
                   <div class="w-full h-full bg-white rounded-l-xl opacity-[0.98] shadow-sm border-y border-l border-[#E2E8F0] flex p-[5px] gap-1.5 pointer-events-none">
                     <div class="w-full h-full bg-[#F1F5F9] rounded border border-[#E2E8F0]"></div>
                   </div>
                </div>
                <!-- Right (Dark) -->
                <div class="w-1/2 h-full bg-[#27272A] pt-1.5 pr-1.5 pb-1.5 pl-[2px]">
                   <div class="w-full h-full bg-[#09090B] rounded-r-xl opacity-[0.98] shadow-[0_1px_3px_rgba(0,0,0,0.5)] border-y border-r border-[#3F3F46] flex p-[5px] pointer-events-none justify-end">
                     <div class="w-full h-full bg-[#09090B] rounded border border-[#3F3F46] flex flex-col gap-1.5 p-1.5">
                       <div class="w-[80%] h-1.5 bg-[#52525B] rounded-full"></div>
                       <div class="w-full h-1 bg-[#27272A] rounded-full mt-1"></div>
                     </div>
                   </div>
                </div>
              </div>
            </button>
            <span class="text-[13px] font-medium text-muted-foreground" :class="{ 'text-foreground': selectedTheme === 'auto' }">Follow System</span>
          </div>
        </div>
      </section>

      <hr class="border-border/30" />

      <!-- Config Section -->
      <section class="space-y-3">
        <div class="flex items-center gap-2">
          <Icon icon="hugeicons:settings-01" class="size-[15px] text-muted-foreground" />
          <h2 class="text-[13px] font-medium text-muted-foreground">Configuration</h2>
        </div>
        <div class="flex items-center justify-between py-2 px-3 rounded-xl border border-border/50 hover:border-border/80 transition-colors">
          <div class="flex items-center gap-3">
            <div class="size-8 rounded-lg bg-muted/40 flex items-center justify-center shrink-0">
              <Icon icon="hugeicons:file-edit" class="size-[16px] text-foreground/70" />
            </div>
            <div>
              <p class="text-[13px] font-medium text-foreground">config.toml</p>
              <p class="text-[12px] text-muted-foreground mt-0.5">Open and edit your Gloamy configuration file</p>
            </div>
          </div>
          <button
            @click="openConfigFile"
            class="flex items-center gap-1.5 px-3 py-1.5 border border-border hover:border-foreground/30 text-foreground transition-colors rounded-lg text-[12px] font-medium shrink-0"
          >
            Open file
            <Icon icon="hugeicons:arrow-up-right-01" class="size-[13px]" />
          </button>
        </div>
      </section>

      <hr class="border-border/30" />

      <!-- Account Section -->
      <section class="space-y-3">
        <div class="flex items-center gap-2">
          <Icon icon="hugeicons:user-account" class="size-[15px] text-muted-foreground" />
          <h2 class="text-[13px] font-medium text-muted-foreground">Account</h2>
        </div>
        <div class="flex items-center justify-between py-2 px-3 rounded-xl border border-border/50 hover:border-border/80 transition-colors">
          <div class="flex items-center gap-3">
            <div class="size-8 rounded-lg bg-muted/40 flex items-center justify-center shrink-0">
              <Icon icon="hugeicons:logout-02" class="size-[16px] text-foreground/70" />
            </div>
            <div>
              <p class="text-[13px] font-medium text-foreground">Sign out</p>
              <p class="text-[12px] text-muted-foreground mt-0.5">Remove this device's pairing token</p>
            </div>
          </div>
          <button
            @click="logout"
            class="flex items-center gap-1.5 px-3 py-1.5 border border-destructive/30 hover:border-destructive/60 text-destructive transition-colors rounded-lg text-[12px] font-medium shrink-0"
          >
            <Icon icon="hugeicons:logout-02" class="size-[13px]" />
            Sign out device
          </button>
        </div>
      </section>

      <hr class="border-border/30" />

      <!-- About Section -->
      <section class="space-y-3">
        <div class="flex items-center gap-2">
          <Icon icon="hugeicons:information-circle" class="size-[15px] text-muted-foreground" />
          <h2 class="text-[13px] font-medium text-muted-foreground">About</h2>
        </div>
        <div class="flex items-center justify-between py-2 px-3 rounded-xl border border-border/50 hover:border-border/80 transition-colors">
          <div class="flex items-center gap-3">
            <div class="size-8 rounded-lg bg-muted/40 flex items-center justify-center shrink-0">
              <Icon icon="hugeicons:globe" class="size-[16px] text-foreground/70" />
            </div>
            <div>
              <p class="text-[13px] font-medium text-foreground">Gloamy website</p>
              <p class="text-[12px] text-muted-foreground mt-0.5">Documentation, updates, and more at gloamy.co</p>
            </div>
          </div>
          <button
            @click="openWebsite"
            class="flex items-center gap-1.5 px-3 py-1.5 border border-border hover:border-foreground/30 text-foreground transition-colors rounded-lg text-[12px] font-medium shrink-0"
          >
            gloamy.co
            <Icon icon="hugeicons:arrow-up-right-01" class="size-[13px]" />
          </button>
        </div>
      </section>

    </div>
  </div>
</template>
