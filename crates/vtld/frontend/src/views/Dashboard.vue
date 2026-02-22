<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { apiFetch } from '../api'

const status = ref<any>(null)
const drives = ref<any[]>([])
const media = ref<any[]>([])
const error = ref('')
let ws: WebSocket | null = null
let pollTimer: number | null = null

async function fetchData() {
  try {
    const [sResp, dResp, mResp] = await Promise.all([
      apiFetch('/api/vtl/status'),
      apiFetch('/api/vtl/drives'),
      apiFetch('/api/vtl/media'),
    ])
    if (sResp.ok) status.value = await sResp.json()
    if (dResp.ok) drives.value = await dResp.json()
    if (mResp.ok) media.value = await mResp.json()
    error.value = ''
  } catch {
    error.value = 'Failed to fetch data'
  }
}

function connectWebSocket() {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
  ws = new WebSocket(`${proto}//${location.host}/api/ws`)
  ws.onmessage = () => { fetchData() }
  ws.onclose = () => {
    ws = null
    pollTimer = window.setTimeout(() => { connectWebSocket() }, 5000)
  }
}

onMounted(() => {
  fetchData()
  connectWebSocket()
})

onUnmounted(() => {
  ws?.close()
  if (pollTimer) clearTimeout(pollTimer)
})
</script>

<template>
  <div>
    <h2>Dashboard</h2>

    <p v-if="error" class="error">{{ error }}</p>

    <section class="card" v-if="status">
      <h3>Library Status</h3>
      <p>Status: <strong>{{ status.status }}</strong></p>
    </section>

    <section class="card">
      <h3>Drives</h3>
      <p v-if="drives.length === 0" class="empty">No drives configured.</p>
      <table v-else>
        <thead><tr><th>ID</th><th>Status</th></tr></thead>
        <tbody>
          <tr v-for="d in drives" :key="d.id">
            <td>{{ d.id }}</td>
            <td>{{ d.status }}</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section class="card">
      <h3>Media</h3>
      <p v-if="media.length === 0" class="empty">No media loaded.</p>
      <table v-else>
        <thead><tr><th>Barcode</th><th>Status</th></tr></thead>
        <tbody>
          <tr v-for="m in media" :key="m.barcode">
            <td>{{ m.barcode }}</td>
            <td>{{ m.status }}</td>
          </tr>
        </tbody>
      </table>
    </section>
  </div>
</template>

<style scoped>
h2 { margin-bottom: 1rem; }
.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.5rem; font-size: 1rem; color: #1a1a2e; }
.empty { color: #888; font-size: 0.9rem; }
.error { color: #c0392b; margin-bottom: 1rem; }
table { width: 100%; border-collapse: collapse; }
th, td { text-align: left; padding: 0.4rem 0.6rem; border-bottom: 1px solid #eee; font-size: 0.9rem; }
th { font-weight: 600; color: #555; }
</style>
