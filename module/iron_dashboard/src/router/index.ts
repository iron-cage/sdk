import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import LoginView from '../views/LoginView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: LoginView,
      meta: { requiresAuth: false },
    },
    {
      path: '/',
      redirect: '/dashboard',
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: () => import('../views/DashboardView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/agents',
      name: 'agents',
      component: () => import('../views/AgentsView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/agents/:agentId/tokens',
      name: 'agent-tokens',
      component: () => import('../views/AgentTokensView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/usage',
      name: 'usage',
      component: () => import('../views/UsageView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/limits',
      name: 'limits',
      component: () => import('../views/LimitsView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/traces',
      name: 'traces',
      component: () => import('../views/TracesView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/providers',
      name: 'providers',
      component: () => import('../views/ProvidersView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/users',
      name: 'users',
      component: () => import('../views/UsersView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/budget-requests',
      name: 'budget-requests',
      component: () => import('../views/BudgetRequestsView.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

// Navigation guard
router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()
  const requiresAuth = to.meta.requiresAuth !== false

  if (requiresAuth && !authStore.isAuthenticated) {
    next('/login')
  } else if (to.path === '/login' && authStore.isAuthenticated) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router
