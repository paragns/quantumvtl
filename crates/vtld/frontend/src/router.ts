import { createRouter, createWebHistory } from 'vue-router'
import Login from './views/Login.vue'
import TabLayout from './views/TabLayout.vue'
import DashboardView from './views/DashboardView.vue'
import DriveDetail from './views/DriveDetail.vue'
import InitiatorsView from './views/InitiatorsView.vue'
import ApiDocs from './views/ApiDocs.vue'
import MediaDetail from './views/MediaDetail.vue'
import DeviceDetail from './views/DeviceDetail.vue'
import CommandDetail from './views/CommandDetail.vue'
import { checkTokenValid } from './api'

const routes = [
  { path: '/login', component: Login, meta: { public: true } },
  {
    path: '/',
    component: TabLayout,
    redirect: '/dashboard',
    children: [
      { path: 'dashboard', name: 'dashboard', component: DashboardView },
      { path: 'ssc/drive/:id', name: 'drive-detail', component: DriveDetail, props: true },
      { path: 'media/:barcode', name: 'media-detail', component: MediaDetail, props: true },
      { path: 'initiators', name: 'initiators', component: InitiatorsView },
    ],
  },
  { path: '/api-docs', component: ApiDocs, meta: { public: true } },
  { path: '/device/changer', component: DeviceDetail },
  { path: '/device/drive/:id', component: DeviceDetail },
  { path: '/device/changer/cmd/:seq', component: CommandDetail },
  { path: '/device/drive/:id/cmd/:seq', component: CommandDetail },
  // Legacy redirects
  { path: '/smc', redirect: '/dashboard' },
  { path: '/ssc', redirect: '/dashboard' },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach(async (to) => {
  if (to.meta.public) return true
  const valid = await checkTokenValid()
  if (!valid) {
    return { path: '/login', query: { redirect: to.fullPath } }
  }
  return true
})
