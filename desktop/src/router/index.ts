import { createMemoryHistory, createRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import PlaceholderPage from '@/pages/placeholder.vue'

const routes = [
  {
    path: '/',
    name: 'dashboard',
    component: () => import('@/pages/dashboard.vue'),
    meta: {
      title: 'Dashboard',
      description: 'Monitor your agent runtime at a glance.',
    },
  },
  {
    path: '/chat',
    name: 'agent-chat',
    component: () => import('@/pages/home.vue'),
    meta: {
      title: 'Agent Chat',
      description: 'Chat directly with your Gloamy agent.',
    },
  },
  {
    path: '/tools',
    name: 'tools',
    component: () => import('@/pages/tools.vue'),
    meta: {
      title: 'Tools',
      description: 'Configure and inspect available tools.',
    },
  },
  {
    path: '/cron-jobs',
    name: 'cron-jobs',
    component: () => import('@/pages/cron-jobs.vue'),
    meta: {
      title: 'Cron Jobs',
      description: 'Manage scheduled automations and routines.',
    },
  },
  {
    path: '/integrations',
    name: 'integrations',
    component: () => import('@/pages/integrations.vue'),
    meta: {
      title: 'Integrations',
      description: 'Connect channels, providers, and peripherals.',
    },
  },
  {
    path: '/memory',
    name: 'memory',
    component: () => import('@/pages/memory.vue'),
    meta: {
      title: 'Memory',
      description: 'Inspect or purge conversation memory stores.',
    },
  },
  {
    path: '/configuration',
    name: 'configuration',
    component: PlaceholderPage,
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
    component: PlaceholderPage,
    meta: {
      title: 'Logs',
      description: 'Trace actions, events, and transport logs.',
    },
  },
  {
    path: '/doctor',
    name: 'doctor',
    component: PlaceholderPage,
    meta: {
      title: 'Doctor',
      description: 'Run diagnostics to catch misconfiguration.',
    },
  },
  {
    path: '/authentication',
    name: 'authentication',
    component: () => import('@/pages/authentication.vue'),
    meta: {
      title: 'Authentication & Pairing',
      description: 'Pair new clients and manage credentials.',
    },
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/pages/settings.vue'),
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
