<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { useColorMode } from '@vueuse/core'
import { open } from '@tauri-apps/plugin-shell'
import { homeDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'

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

type PermissionStatus = 'granted' | 'denied' | 'unknown'

interface Permission {
  id: string
  icon: string
  name: string
  description: string
  settingsUrl: string
  status: PermissionStatus
}

const showPermissionsModal = ref(false)
const checking = ref(false)

const permissions = ref<Permission[]>([
  {
    id: 'accessibility',
    icon: 'hugeicons:computer-terminal-01',
    name: 'Accessibility',
    description: 'UI automation — clicking, typing, reading on-screen elements.',
    settingsUrl: 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility',
    status: 'unknown',
  },
  {
    id: 'screen-recording',
    icon: 'hugeicons:eye',
    name: 'Screen Recording',
    description: 'Capture screenshots and screen content for computer use.',
    settingsUrl: 'x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture',
    status: 'unknown',
  },
  {
    id: 'automation',
    icon: 'hugeicons:robot-01',
    name: 'Automation',
    description: 'Control apps via AppleScript and JXA — Reminders, Calendar, Finder.',
    settingsUrl: 'x-apple.systempreferences:com.apple.preference.security?Privacy_Automation',
    status: 'unknown',
  },
  {
    id: 'full-disk-access',
    icon: 'hugeicons:hard-drive',
    name: 'Full Disk Access',
    description: 'Read and write files across the filesystem, including protected locations.',
    settingsUrl: 'x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles',
    status: 'unknown',
  },
])

async function refreshPermissions() {
  checking.value = true
  try {
    const [accessibility, screenRecording] = await Promise.all([
      invoke<boolean>('check_accessibility_permission').catch(() => null),
      invoke<boolean>('check_screen_recording_permission').catch(() => null),
    ])
    permissions.value = permissions.value.map(p => {
      if (p.id === 'accessibility' && accessibility !== null) {
        return { ...p, status: (accessibility ? 'granted' : 'denied') as PermissionStatus }
      }
      if (p.id === 'screen-recording' && screenRecording !== null) {
        return { ...p, status: (screenRecording ? 'granted' : 'denied') as PermissionStatus }
      }
      return p
    })
  }
  finally {
    checking.value = false
  }
}

async function openPermissionsModal() {
  showPermissionsModal.value = true
  await refreshPermissions()
}

function statusDotClass(status: PermissionStatus): string {
  if (status === 'granted') return 'bg-emerald-500'
  return 'bg-muted-foreground/30'
}

function openPermissionSettings(url: string) {
  open(url)
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
        <div class="flex items-center justify-between py-2 px-1">
          <div class="flex items-center gap-3">
            <div class="size-8 flex items-center justify-center shrink-0">
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

      <!-- Permissions Section -->
      <section class="space-y-3">
        <div class="flex items-center gap-2">
          <Icon icon="hugeicons:shield-01" class="size-[15px] text-muted-foreground" />
          <h2 class="text-[13px] font-medium text-muted-foreground">Permissions</h2>
        </div>
        <div class="flex items-center justify-between py-2 px-1">
          <div class="flex items-center gap-3">
            <div class="size-8 flex items-center justify-center shrink-0">
              <Icon icon="hugeicons:shield-key" class="size-[16px] text-foreground/70" />
            </div>
            <div>
              <p class="text-[13px] font-medium text-foreground">System Permissions</p>
              <p class="text-[12px] text-muted-foreground mt-0.5">macOS permissions required for computer use and automation</p>
            </div>
          </div>
          <button
            @click="openPermissionsModal"
            class="flex items-center gap-1.5 px-3 py-1.5 border border-border hover:border-foreground/30 text-foreground transition-colors rounded-lg text-[12px] font-medium shrink-0"
          >
            Manage
            <Icon icon="hugeicons:arrow-right-01" class="size-[13px]" />
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
        <div class="flex items-center justify-between py-2 px-1">
          <div class="flex items-center gap-3">
            <div class="size-8 flex items-center justify-center shrink-0">
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
        <div class="flex items-center justify-between py-2 px-1">
          <div class="flex items-center gap-3">
            <div class="size-8 flex items-center justify-center shrink-0">
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

  <Teleport to="body">
    <div
      v-if="showPermissionsModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-[1.5px] px-4"
      @click.self="showPermissionsModal = false"
    >
      <div class="w-full max-w-lg rounded-[20px] border border-border/35 bg-background shadow-[0_20px_55px_-28px_rgba(0,0,0,0.72)] motion-safe:animate-in motion-safe:fade-in-0 motion-safe:zoom-in-95 motion-safe:duration-200 motion-safe:ease-out">

        <!-- Modal header -->
        <div class="flex items-center justify-between px-6 py-5 border-b border-border/30">
          <div class="flex items-center gap-2">
            <Icon icon="hugeicons:shield-key" class="size-[15px] text-muted-foreground" />
            <h3 class="text-[14px] font-semibold text-foreground">System Permissions</h3>
          </div>
          <div class="flex items-center gap-1">
            <button
              @click="refreshPermissions"
              class="p-1.5 rounded-lg hover:bg-muted/40 transition-colors text-muted-foreground hover:text-foreground"
              title="Refresh status"
            >
              <Icon icon="hugeicons:refresh" class="size-3.5" :class="{ 'animate-spin': checking }" />
            </button>
            <button
              @click="showPermissionsModal = false"
              class="p-1.5 rounded-lg hover:bg-muted/40 transition-colors text-muted-foreground hover:text-foreground"
            >
              <Icon icon="hugeicons:cancel-01" class="size-4" />
            </button>
          </div>
        </div>

        <!-- Permission rows -->
        <div class="px-6 py-5 space-y-6">
          <div
            v-for="perm in permissions"
            :key="perm.id"
            class="flex items-center justify-between"
          >
            <div class="flex items-center gap-3 min-w-0">
              <div class="size-8 flex items-center justify-center shrink-0">
                <Icon :icon="perm.icon" class="size-[15px] text-foreground/70" />
              </div>
              <div class="min-w-0">
                <p class="text-[13px] font-medium text-foreground">{{ perm.name }}</p>
                <p class="text-[11px] text-muted-foreground mt-0.5 leading-relaxed">{{ perm.description }}</p>
              </div>
            </div>
            <div class="flex items-center gap-2.5 shrink-0 ml-4">
              <span class="size-1.5 rounded-full" :class="statusDotClass(perm.status)" />
              <button
                @click="openPermissionSettings(perm.settingsUrl)"
                class="flex items-center gap-1 px-2.5 py-1 text-[11px] font-medium border border-border/60 rounded-lg hover:border-foreground/30 text-foreground transition-colors whitespace-nowrap"
              >
                Open
                <Icon icon="hugeicons:arrow-up-right-01" class="size-[11px]" />
              </button>
            </div>
          </div>
        </div>

        <!-- Footer note -->
        <div class="px-6 pb-6 pt-2">
          <p class="text-[11px] text-muted-foreground leading-relaxed">
            Refresh after granting. Accessibility changes require a daemon restart.
          </p>
        </div>

      </div>
    </div>
  </Teleport>

</template>
