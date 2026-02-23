<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const showNav = computed(() => route.path !== '/login')

const isTabRoute = computed(() => {
  const name = route.name as string | undefined
  return name === 'smc' || name === 'ssc' || name === 'drive-detail' || name === 'initiators'
})

const activeTab = computed(() => {
  const name = route.name as string
  if (name === 'drive-detail' || name === 'ssc') return 'ssc'
  if (name === 'smc') return 'smc'
  if (name === 'initiators') return 'initiators'
  return ''
})

function logout() {
  localStorage.removeItem('token')
  router.push('/login')
}
</script>

<template>
  <div id="layout">
    <header v-if="showNav">
      <nav>
        <router-link to="/" class="brand">QuantumVTL</router-link>
        <template v-if="isTabRoute">
          <router-link to="/smc" class="tab" :class="{ active: activeTab === 'smc' }">Changer</router-link>
          <router-link to="/ssc" class="tab" :class="{ active: activeTab === 'ssc' }">Drives</router-link>
          <router-link to="/initiators" class="tab" :class="{ active: activeTab === 'initiators' }">Initiators</router-link>
        </template>
        <span class="spacer"></span>
        <router-link to="/api-docs" class="right-link">API Docs</router-link>
        <a href="#" class="right-link" @click.prevent="logout">Logout</a>
      </nav>
    </header>
    <main>
      <router-view />
    </main>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; color: #333; }
#layout { min-height: 100vh; display: flex; flex-direction: column; }
header { background: #1a1a2e; color: #fff; padding: 0 1rem; }
nav { display: flex; align-items: center; height: 3rem; gap: 0; }
nav .brand { color: #fff; font-weight: bold; font-size: 1rem; text-decoration: none; margin-right: 1.5rem; }
nav .tab {
  color: #999; text-decoration: none; font-size: 0.88rem; font-weight: 600;
  padding: 0 0.85rem; height: 3rem; display: flex; align-items: center;
  border-bottom: 2px solid transparent; transition: color 0.15s, border-color 0.15s;
}
nav .tab:hover { color: #ddd; }
nav .tab.active { color: #fff; border-bottom-color: #fff; }
nav .right-link { color: #ccc; text-decoration: none; font-size: 0.9rem; margin-left: 1rem; }
nav .right-link:hover { color: #fff; }
.spacer { flex: 1; }
main { flex: 1; padding: 1.5rem; max-width: 1200px; margin: 0 auto; width: 100%; }
</style>
