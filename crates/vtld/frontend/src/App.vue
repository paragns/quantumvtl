<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const showNav = computed(() => route.path !== '/login')

const activeTab = computed(() => {
  const name = route.name as string
  if (name === 'devices' || name === 'drive-detail') return 'devices'
  if (name === 'media' || name === 'media-detail') return 'media'
  if (name === 'initiators') return 'initiators'
  if (name === 'config') return 'config'
  if (name === 'docs') return 'docs'
  if (name === 'api-docs') return 'api-docs'
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
        <router-link to="/devices" class="tab" :class="{ active: activeTab === 'devices' }">Devices</router-link>
        <router-link to="/media" class="tab" :class="{ active: activeTab === 'media' }">Media</router-link>
        <router-link to="/initiators" class="tab" :class="{ active: activeTab === 'initiators' }">Initiators</router-link>
        <router-link to="/config" class="tab" :class="{ active: activeTab === 'config' }">Config</router-link>
        <span class="spacer"></span>
        <router-link to="/docs" class="tab right-tab" :class="{ active: activeTab === 'docs' }">Docs</router-link>
        <router-link to="/api-docs" class="tab right-tab" :class="{ active: activeTab === 'api-docs' }">API</router-link>
        <a href="#" class="tab right-tab" @click.prevent="logout">Logout</a>
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
nav .tab.right-tab { color: #aaa; }
nav .tab.right-tab:hover { color: #ddd; }
nav .tab.right-tab.active { color: #fff; border-bottom-color: #fff; }
.spacer { flex: 1; }
main { flex: 1; padding: 1.5rem; max-width: 1400px; margin: 0 auto; width: 100%; }
</style>
