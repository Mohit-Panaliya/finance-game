import { createRouter, createWebHistory } from '@ionic/vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { getToken } from '@/api/client'
import { isEntityType } from '@/entityConfig'

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/dashboard' },
  {
    path: '/dashboard',
    name: 'dashboard',
    component: () => import('@/pages/DashboardPage.vue'),
    meta: { tab: 'dashboard', title: 'Dashboard' }
  },
  {
    path: '/accounts',
    name: 'accounts',
    component: () => import('@/pages/AccountsPage.vue'),
    meta: { tab: 'accounts', title: 'Accounts' }
  },
  {
    path: '/accounts/:entity',
    name: 'account-detail',
    component: () => import('@/pages/AccountDetailPage.vue'),
    meta: { tab: 'accounts', title: 'Records' }
  },
  {
    path: '/accounts/:entity/new',
    name: 'account-new',
    component: () => import('@/pages/AccountDetailPage.vue'),
    meta: { tab: 'accounts', title: 'New record' }
  },
  // Full statement pages. The backend generates UUID primary keys, so `:id` is
  // pinned to that shape: it can never swallow the literal `new` segment of the
  // create form (a plain `:id` loses that tie in vue-router's scoring).
  {
    path: '/accounts/banks/:id([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})',
    name: 'bank-statement',
    component: () => import('@/pages/AccountStatementPage.vue'),
    meta: { tab: 'accounts', title: 'Statement' }
  },
  {
    path: '/accounts/credit-cards/:id([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})',
    name: 'card-statement',
    component: () => import('@/pages/AccountStatementPage.vue'),
    meta: { tab: 'accounts', title: 'Statement' }
  },
  {
    path: '/transactions',
    name: 'transactions',
    component: () => import('@/pages/TransactionsPage.vue'),
    meta: { tab: 'transactions', title: 'Activity' }
  },
  {
    path: '/analytics',
    name: 'analytics',
    component: () => import('@/pages/AnalyticsPage.vue'),
    meta: { tab: 'analytics', title: 'Insights' }
  },
  {
    path: '/debts',
    name: 'debts',
    component: () => import('@/pages/DebtsPage.vue'),
    meta: { tab: 'debts', title: 'Debts' }
  },
  {
    path: '/notes',
    name: 'notes',
    component: () => import('@/pages/NotesPage.vue'),
    meta: { tab: 'notes', title: 'Notes' }
  },
  {
    path: '/buy-list',
    name: 'buy-list',
    component: () => import('@/pages/BuyListPage.vue'),
    meta: { tab: 'buy-list', title: 'Buy List' }
  },


  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/pages/SettingsPage.vue'),
    meta: { tab: 'settings', title: 'Settings' }
  },
  {
    path: '/login',
    name: 'login',
    component: () => import('@/pages/LoginPage.vue'),
    meta: { public: true, title: 'Sign in' }
  },
  {
    path: '/register',
    name: 'register',
    component: () => import('@/pages/RegisterPage.vue'),
    meta: { public: true, title: 'Create account' }
  },
  {
    path: '/assets/:id/depreciation',
    name: 'asset-depreciation',
    component: () => import('@/pages/DepreciationPage.vue'),
    meta: { tab: 'assets', title: 'Depreciation' }
  },
  { path: '/:pathMatch(.*)*', redirect: '/dashboard' }
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes
})

const APP_TITLE = 'FinTrack'

router.beforeEach((to) => {
  // Read the persisted token directly: `auth.user` is still null on a hard refresh
  // until /auth/me resolves, which would bounce signed-in users back to /login.
  const token = getToken()

  if (!to.meta.public && !token) {
    return { name: 'login', query: { redirect: to.fullPath } }
  }
  if (to.meta.public && token) {
    return { name: 'dashboard' }
  }
  if (typeof to.params.entity === 'string' && !isEntityType(to.params.entity)) {
    return { name: 'accounts' }
  }
  return true
})

router.afterEach((to) => {
  const title = (to.meta?.title as string | undefined) ?? ''
  document.title = title ? `${title} · ${APP_TITLE}` : APP_TITLE
})

export default router