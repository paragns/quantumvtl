import { createRouter, createWebHistory } from 'vue-router'
import Login from './views/Login.vue'
import TabLayout from './views/TabLayout.vue'
import DevicesView from './views/DevicesView.vue'
import MediaView from './views/MediaView.vue'
import DriveDetail from './views/DriveDetail.vue'
import InitiatorsView from './views/InitiatorsView.vue'
import ConfigView from './views/ConfigView.vue'
import DocsView from './views/DocsView.vue'
import ApiDocs from './views/ApiDocs.vue'
import MediaDetail from './views/MediaDetail.vue'
import DeviceDetail from './views/DeviceDetail.vue'
import CommandDetail from './views/CommandDetail.vue'
import SessionCommandDetail from './views/SessionCommandDetail.vue'
import { checkTokenValid } from './api'

const routes = [
  { path: '/login', component: Login, meta: { public: true } },
  {
    path: '/',
    component: TabLayout,
    redirect: '/devices',
    children: [
      { path: 'devices', name: 'devices', component: DevicesView },
      { path: 'media', name: 'media', component: MediaView },
      { path: 'media/:barcode', name: 'media-detail', component: MediaDetail, props: true },
      { path: 'initiators', name: 'initiators', component: InitiatorsView },
      { path: 'config', name: 'config', component: ConfigView },
      { path: 'docs', name: 'docs', component: DocsView },
      { path: 'api-docs', name: 'api-docs', component: ApiDocs },
      { path: 'ssc/drive/:id', name: 'drive-detail', component: DriveDetail, props: true },
    ],
  },
  { path: '/device/changer', component: DeviceDetail },
  { path: '/device/drive/:id', component: DeviceDetail },
  { path: '/device/changer/cmd/:seq', component: CommandDetail },
  { path: '/device/drive/:id/cmd/:seq', component: CommandDetail },
  { path: '/initiators/:tsih/cmd/:seq', component: SessionCommandDetail },
  // Legacy redirects
  { path: '/smc', redirect: '/devices' },
  { path: '/ssc', redirect: '/devices' },
  { path: '/dashboard', redirect: '/devices' },
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
