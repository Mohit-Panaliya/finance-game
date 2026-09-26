import { createRouter, createWebHistory } from '@ionic/vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/authStore'

const routes: RouteRecordRaw[] = [
  { path: '/login', name: 'login', component: () => import('@/pages/LoginPage.vue'), meta: { public: true } },
  { path: '/register', name: 'register', component: () => import('@/pages/RegisterPage.vue'), meta: { public: true } },
  { path: '/', name: 'village', component: () => import('@/pages/VillagePage.vue') },
  { path: '/battle', name: 'battle', component: () => import('@/pages/BattlePage.vue') },
  { path: '/records', name: 'records', component: () => import('@/pages/RecordsPage.vue') },
  { path: '/records/:entity/new', name: 'record-new', component: () => import('@/pages/RecordEntityPage.vue'), props: { startNew: true } },
  { path: '/records/:entity', name: 'record-entity', component: () => import('@/pages/RecordEntityPage.vue') },
  { path: '/army', name: 'army', component: () => import('@/pages/ArmyPage.vue') },
  { path: '/achievements', name: 'achievements', component: () => import('@/pages/AchievementsPage.vue') },
  { path: '/leaderboard', name: 'leaderboard', component: () => import('@/pages/LeaderboardPage.vue') },
  { path: '/settings', name: 'settings', component: () => import('@/pages/SettingsPage.vue') },
  { path: '/analysis', name: 'analysis', component: () => import('@/pages/AnalysisPage.vue') },
  { path: '/:pathMatch(.*)*', redirect: '/' }
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (!auth.ready) await auth.init()
  if (to.meta.public) {
    if (auth.isAuthenticated && (to.name === 'login' || to.name === 'register')) return '/'
    return true
  }
  if (!auth.isAuthenticated) return { name: 'login', query: { redirect: to.fullPath } }
  return true
})

export default router
