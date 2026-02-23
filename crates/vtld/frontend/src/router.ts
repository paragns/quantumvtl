import { createRouter, createWebHistory } from 'vue-router'
import Login from './views/Login.vue'
import TabLayout from './views/TabLayout.vue'
import SmcView from './views/SmcView.vue'
import SscView from './views/SscView.vue'
import DriveDetail from './views/DriveDetail.vue'
import InitiatorsView from './views/InitiatorsView.vue'
import ApiDocs from './views/ApiDocs.vue'
import { checkTokenValid } from './api'

const routes = [
  { path: '/login', component: Login, meta: { public: true } },
  {
    path: '/',
    component: TabLayout,
    redirect: '/smc',
    children: [
      { path: 'smc', name: 'smc', component: SmcView },
      { path: 'ssc', name: 'ssc', component: SscView },
      { path: 'ssc/drive/:id', name: 'drive-detail', component: DriveDetail, props: true },
      { path: 'initiators', name: 'initiators', component: InitiatorsView },
    ],
  },
  { path: '/api-docs', component: ApiDocs, meta: { public: true } },
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
