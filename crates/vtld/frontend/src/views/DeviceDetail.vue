<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { fetchScsiLog, type ScsiLogSummary } from '../api'

const route = useRoute()
const router = useRouter()

const deviceType = computed(() => {
  return route.path.startsWith('/device/changer') ? 'changer' : 'drive'
})
const deviceId = computed(() => {
  if (deviceType.value === 'changer') return 0
  return Number(route.params.id) || 0
})
const deviceLabel = computed(() => {
  if (deviceType.value === 'changer') return 'Media Changer'
  return `Tape Drive ${deviceId.value}`
})

const entries = ref<ScsiLogSummary[]>([])
const error = ref('')
let ws: WebSocket | null = null
let pollTimer: number | null = null
let destroyed = false
let fetching = false

async function loadData() {
  const resp = await fetchScsiLog(deviceType.value, deviceId.value, 20)
  if (resp) {
    entries.value = resp.entries
    error.value = ''
  } else {
    error.value = 'Failed to fetch SCSI log'
  }
}

function guardedLoad() {
  if (fetching) return
  fetching = true
  loadData().finally(() => { fetching = false })
}

function connectWebSocket() {
  if (destroyed) return
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
  ws = new WebSocket(`${proto}//${location.host}/api/ws`)
  ws.onmessage = () => { guardedLoad() }
  ws.onclose = () => {
    ws = null
    if (!destroyed) {
      pollTimer = window.setTimeout(() => { connectWebSocket() }, 5000)
    }
  }
}

function cmdPath(entry: ScsiLogSummary): string {
  if (deviceType.value === 'changer') return `/device/changer/cmd/${entry.seq}`
  return `/device/drive/${deviceId.value}/cmd/${entry.seq}`
}

function formatTime(ts: string): string {
  try {
    const d = new Date(ts)
    const hh = String(d.getHours()).padStart(2, '0')
    const mm = String(d.getMinutes()).padStart(2, '0')
    const ss = String(d.getSeconds()).padStart(2, '0')
    const ms = String(d.getMilliseconds()).padStart(3, '0')
    return `${hh}:${mm}:${ss}.${ms}`
  } catch { return ts }
}

function formatDuration(us: number): string {
  if (us < 1000) return `${us}us`
  if (us < 1000000) return `${(us / 1000).toFixed(1)}ms`
  return `${(us / 1000000).toFixed(2)}s`
}

function formatOpcode(op: number): string {
  return `0x${op.toString(16).toUpperCase().padStart(2, '0')}`
}

onMounted(() => {
  guardedLoad()
  connectWebSocket()
})

onUnmounted(() => {
  destroyed = true
  if (pollTimer) clearTimeout(pollTimer)
  ws?.close()
})
</script>

<template>
  <div class="device-detail">
    <nav class="breadcrumb">
      <router-link to="/">Dashboard</router-link>
      <span class="sep">/</span>
      <span>{{ deviceLabel }}</span>
    </nav>

    <h2>{{ deviceLabel }} — SCSI Activity</h2>

    <p v-if="error" class="error">{{ error }}</p>

    <section class="card" v-if="entries.length > 0">
      <table class="scsi-table">
        <thead>
          <tr>
            <th>Seq</th>
            <th>Timestamp</th>
            <th>Opcode</th>
            <th>Command</th>
            <th>Status</th>
            <th>Duration</th>
            <th>Out</th>
            <th>In</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="e in entries"
            :key="e.seq"
            class="clickable"
            @click="router.push(cmdPath(e))"
          >
            <td class="mono">{{ e.seq }}</td>
            <td class="mono">{{ formatTime(e.timestamp) }}</td>
            <td class="mono">{{ formatOpcode(e.opcode) }}</td>
            <td class="cmd-name">{{ e.opcode_name }}</td>
            <td>
              <span class="status-badge" :class="{ good: e.status === 0, error: e.status !== 0 }">
                {{ e.status_name }}
              </span>
            </td>
            <td class="mono">{{ formatDuration(e.duration_us) }}</td>
            <td class="mono">{{ e.data_out_len }}</td>
            <td class="mono">{{ e.data_in_len }}</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section class="card empty" v-else-if="!error">
      <p>No SCSI commands recorded yet.</p>
    </section>
  </div>
</template>

<style scoped>
.device-detail { max-width: 1200px; margin: 0 auto; }
.breadcrumb { font-size: 0.85rem; margin-bottom: 0.75rem; }
.breadcrumb a { color: #555; text-decoration: none; }
.breadcrumb a:hover { color: #1a1a2e; text-decoration: underline; }
.sep { margin: 0 0.4rem; color: #aaa; }
h2 { margin-bottom: 1rem; }
.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.error { color: #c0392b; margin-bottom: 1rem; }
.empty p { color: #888; font-style: italic; }

.scsi-table { width: 100%; border-collapse: collapse; font-size: 0.82rem; }
.scsi-table th { text-align: left; padding: 0.4rem 0.5rem; border-bottom: 2px solid #e0e0e0; font-size: 0.75rem; color: #888; text-transform: uppercase; }
.scsi-table td { padding: 0.35rem 0.5rem; border-bottom: 1px solid #f0f0f0; }
.scsi-table .clickable { cursor: pointer; }
.scsi-table .clickable:hover { background: #f8f9fa; }
.mono { font-family: 'SF Mono', 'Consolas', 'Liberation Mono', monospace; }
.cmd-name { font-weight: 600; color: #1a1a2e; }
.status-badge { display: inline-block; padding: 0.1rem 0.4rem; border-radius: 3px; font-size: 0.75rem; font-weight: 600; }
.status-badge.good { background: #d4edda; color: #155724; }
.status-badge.error { background: #f8d7da; color: #721c24; }
</style>
