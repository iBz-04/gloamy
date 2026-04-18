import { createMemoryHistory, createRouter } from 'vue-router'
import AuthenticationPage from '@/pages/authentication.vue'
import ConfigurationPage from '@/pages/configuration.vue'
import CronJobsPage from '@/pages/cron-jobs.vue'
import DashboardPage from '@/pages/dashboard.vue'
import DoctorPage from '@/pages/doctor.vue'
import IntegrationsPage from '@/pages/integrations.vue'
import LogsPage from '@/pages/logs.vue'
import MemoryPage from '@/pages/memory.vue'
import PlaceholderPage from '@/pages/placeholder.vue'
import SettingsPage from '@/pages/settings.vue'
import ToolsPage from '@/pages/tools.vue'
import { useAuthStore } from '@/stores/auth'

const routes = [
  {
    path: '/',
    name: 'dashboard',
    component: DashboardPage,
    meta: {
      title: 'Dashboard',
      description: 'Monitor your agent runtime at a glance.',
    },
  },
  {
    path: '/tools',
    name: 'tools',
    component: ToolsPage,
    meta: {
      title: 'Tools',
      description: 'Configure and inspect available tools.',
    },
  },
  {
    path: '/cron-jobs',
    name: 'cron-jobs',
    component: CronJobsPage,
    meta: {
      title: 'Cron Jobs',
      description: 'Manage scheduled automations and routines.',
    },
  },
  {
    path: '/integrations',
    name: 'integrations',
    component: IntegrationsPage,
    meta: {
      title: 'Integrations',
      description: 'Connect channels, providers, and peripherals.',
    },
  },
  {
    path: '/memory',
    name: 'memory',
    component: MemoryPage,
    meta: {
      title: 'Memory',
      description: 'Inspect or purge conversation memory stores.',
    },
  },
  {
    path: '/configuration',
    name: 'configuration',
    component: ConfigurationPage,
    meta: {
      title: 'Configuration',
      description: 'Review runtime configuration and overrides.',
    },
  },
  {
    path: '/cost-tracking',
    name: 'cost-tracking',
    component: PlaceholderPage,
    meta: {
      title: 'Cost Tracking',
      description: 'Track token spend and billing per provider.',
    },
  },
  {
    path: '/logs',
    name: 'logs',
    component: LogsPage,
    meta: {
      title: 'Logs',
      description: 'System logs and debugging.',
    },
  },
  {
    path: '/doctor',
    name: 'doctor',
    component: DoctorPage,
    meta: {
      title: 'Doctor',
      description: 'Run diagnostics to catch misconfiguration.',
    },
  },
  {
    path: '/authentication',
    name: 'authentication',
    component: AuthenticationPage,
    meta: {
      title: 'Authentication & Pairing',
      description: 'Pair new clients and manage credentials.',
    },
  },
  {
    path: '/settings',
    name: 'settings',
    component: SettingsPage,
    meta: {
      title: 'Settings & Theme',
      description: 'Adjust appearance and global preferences.',
    },
  },
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (!auth.isLoaded)
    await auth.load()

  const isAuthRoute = to.path === '/authentication'

  if (!auth.isAuthenticated && !isAuthRoute) {
    return { path: '/authentication', query: { redirect: to.fullPath } }
  }

  if (auth.isAuthenticated && isAuthRoute) {
    // Allow authenticated users to visit the auth page (e.g. to logout or change token)
    return true
  }

  return true
})

export default router
