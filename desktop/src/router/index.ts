import { createMemoryHistory, createRouter } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/pages/home.vue'),
  },
  {
    path: '/search',
    name: 'search',
    component: () => import('@/pages/search.vue'),
  },
  {
    path: '/agents',
    name: 'agents',
    component: () => import('@/pages/agents.vue'),
  },
  {
    path: '/library',
    name: 'library',
    component: () => import('@/pages/library.vue'),
  },
  {
    path: '/canvas',
    name: 'canvas',
    component: () => import('@/pages/canvas.vue'),
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/pages/settings.vue'),
  },
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

export default router
