<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const showNav = computed(() => route.path !== '/login')

function logout() {
  localStorage.removeItem('token')
  router.push('/login')
}
</script>

<template>
  <div id="layout">
    <header v-if="showNav">
      <nav>
        <router-link to="/">QuantumVTL</router-link>
        <span class="spacer"></span>
        <router-link to="/api-docs">API Docs</router-link>
        <a href="#" @click.prevent="logout">Logout</a>
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
nav { display: flex; align-items: center; height: 3rem; gap: 1rem; }
nav a { color: #ccc; text-decoration: none; font-size: 0.9rem; }
nav a:first-child { color: #fff; font-weight: bold; font-size: 1rem; }
nav a:hover { color: #fff; }
.spacer { flex: 1; }
main { flex: 1; padding: 1.5rem; max-width: 1200px; margin: 0 auto; width: 100%; }
</style>
