<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { fetchSimulationSpeed, setSimulationSpeed } from './api'

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

// --- Simulation Speed Slider ---

const speedLabel = ref('Instant')
const sliderPos = ref(100)
let ws: WebSocket | null = null
let wsTimer: number | null = null
let destroyed = false

// Logarithmic mapping: pos 0 = 0.1x, pos 50 = 1.0x, pos 100 = Infinity
function posToFactor(pos: number): number {
  if (pos >= 100) return Infinity
  return Math.pow(10, (pos - 50) / 15)
}

function factorToPos(factor: number): number {
  if (!isFinite(factor) || factor > 1_000_000) return 100
  if (factor <= 0) return 0
  const pos = 15 * Math.log10(factor) + 50
  return Math.max(0, Math.min(100, Math.round(pos)))
}

async function loadSpeed() {
  const resp = await fetchSimulationSpeed()
  if (resp) {
    speedLabel.value = resp.label
    sliderPos.value = factorToPos(resp.speed_factor)
  }
}

async function onSliderChange() {
  const factor = posToFactor(sliderPos.value)
  const resp = await setSimulationSpeed(factor)
  if (resp) {
    speedLabel.value = resp.label
  }
}

function connectSpeedWs() {
  if (destroyed) return
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
  ws = new WebSocket(`${proto}//${location.host}/api/ws`)
  ws.onmessage = () => { loadSpeed() }
  ws.onclose = () => {
    ws = null
    if (!destroyed) {
      wsTimer = window.setTimeout(() => { connectSpeedWs() }, 5000)
    }
  }
}

onMounted(() => {
  loadSpeed()
  connectSpeedWs()
})

onUnmounted(() => {
  destroyed = true
  if (wsTimer) clearTimeout(wsTimer)
  ws?.close()
})
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
        <div class="speed-control">
          <span class="speed-icon" title="Robot simulation speed">&#9889;</span>
          <span class="speed-label-text">Glacial</span>
          <input
            type="range"
            class="speed-slider"
            min="0"
            max="100"
            step="1"
            v-model.number="sliderPos"
            @change="onSliderChange"
            title="Robot simulation speed"
          />
          <span class="speed-label-text">Ludicrous</span>
          <span class="speed-value">{{ speedLabel }}</span>
        </div>
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

/* Speed control slider */
.speed-control {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-right: 0.5rem;
  padding: 0 0.5rem;
}
.speed-icon {
  font-size: 1rem;
  color: #f0c040;
}
.speed-label-text {
  font-size: 0.7rem;
  color: #888;
  white-space: nowrap;
}
.speed-slider {
  width: 100px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: #444;
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}
.speed-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #f0c040;
  cursor: pointer;
  border: none;
}
.speed-slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #f0c040;
  cursor: pointer;
  border: none;
}
.speed-value {
  font-size: 0.75rem;
  color: #f0c040;
  font-weight: 600;
  min-width: 4.5rem;
  text-align: center;
  white-space: nowrap;
}
</style>
