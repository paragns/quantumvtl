import { createRouter, createWebHistory } from 'vue-router'
import Login from './views/Login.vue'
import Dashboard from './views/Dashboard.vue'
import ApiDocs from './views/ApiDocs.vue'
import { checkTokenValid } from './api'

const routes = [
  { path: '/login', component: Login, meta: { public: true } },
  { path: '/', component: Dashboard },
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
