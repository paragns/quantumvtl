<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { apiFetch } from '../api'

interface SlotData {
  address: number
  full: boolean
  barcode: string | null
  source_element: number
}

interface DriveData {
  id: number
  status: string
  serial: string
  barcode: string | null
  position: number
  record_count: number
}

interface StatusData {
  status: string
  vendor: string
  product: string
  serial: string
  total_slots: number
  used_slots: number
  total_drives: number
  import_export_slots: number
}

interface MediaData {
  barcode: string
  location: string
  location_address: number
}

interface Snapshot {
  status: StatusData
  drives: DriveData[]
  slots: SlotData[]
  media: MediaData[]
}

const snapshot = ref<Snapshot | null>(null)
const error = ref('')
let ws: WebSocket | null = null
let pollTimer: number | null = null

const slotsUsed = computed(() => snapshot.value?.status.used_slots ?? 0)
const slotsTotal = computed(() => snapshot.value?.status.total_slots ?? 0)
const utilizationPct = computed(() =>
  slotsTotal.value > 0 ? Math.round((slotsUsed.value / slotsTotal.value) * 100) : 0
)

async function fetchData() {
  try {
    const resp = await apiFetch('/api/vtl/snapshot')
    if (resp.ok) {
      snapshot.value = await resp.json()
      error.value = ''
    } else {
      error.value = `API error: ${resp.status}`
    }
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
  <div class="dashboard">
    <h2>Library Dashboard</h2>

    <p v-if="error" class="error">{{ error }}</p>

    <!-- Summary Card -->
    <section class="card summary" v-if="snapshot">
      <div class="summary-header">
        <div>
          <h3>{{ snapshot.status.vendor }} {{ snapshot.status.product }}</h3>
          <p class="serial">S/N: {{ snapshot.status.serial }}</p>
        </div>
        <span class="badge" :class="snapshot.status.status">{{ snapshot.status.status }}</span>
      </div>
      <div class="stats">
        <div class="stat">
          <span class="stat-value">{{ snapshot.status.total_drives }}</span>
          <span class="stat-label">Drives</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ slotsUsed }} / {{ slotsTotal }}</span>
          <span class="stat-label">Slots Used</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ utilizationPct }}%</span>
          <span class="stat-label">Utilization</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ snapshot.media.length }}</span>
          <span class="stat-label">Media</span>
        </div>
      </div>
    </section>

    <!-- Drive Bays -->
    <section class="card" v-if="snapshot">
      <h3>Drive Bays</h3>
      <div class="drive-row">
        <div
          v-for="d in snapshot.drives"
          :key="d.id"
          class="drive-bay"
          :class="{ loaded: d.status === 'loaded' }"
        >
          <div class="drive-header">
            <span class="drive-id">Drive {{ d.id }}</span>
            <span class="drive-dot" :class="d.status"></span>
          </div>
          <div class="drive-serial">{{ d.serial }}</div>
          <div v-if="d.status === 'loaded'" class="drive-media">
            <span class="drive-barcode">{{ d.barcode }}</span>
            <span class="drive-pos">pos {{ d.position }} / {{ d.record_count }} rec</span>
          </div>
          <div v-else class="drive-empty">Empty</div>
        </div>
      </div>
    </section>

    <!-- Slot Grid -->
    <section class="card" v-if="snapshot">
      <h3>Storage Slots</h3>
      <div class="slot-grid">
        <div
          v-for="s in snapshot.slots"
          :key="s.address"
          class="slot-cell"
          :class="{ full: s.full }"
          :title="s.full ? s.barcode ?? '' : `Slot ${s.address} (empty)`"
        >
          <span class="slot-addr">{{ s.address }}</span>
          <span v-if="s.full" class="slot-barcode">{{ s.barcode }}</span>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.dashboard { max-width: 1200px; margin: 0 auto; }
h2 { margin-bottom: 1rem; }
.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }
.error { color: #c0392b; margin-bottom: 1rem; }

/* Summary */
.summary-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1rem; }
.summary-header h3 { margin-bottom: 0.15rem; font-size: 1.1rem; }
.serial { color: #888; font-size: 0.85rem; }
.badge { display: inline-block; padding: 0.2rem 0.7rem; border-radius: 999px; font-size: 0.8rem; font-weight: 600; text-transform: uppercase; }
.badge.online { background: #d4edda; color: #155724; }
.badge.offline { background: #f8d7da; color: #721c24; }
.stats { display: flex; gap: 2rem; flex-wrap: wrap; }
.stat { display: flex; flex-direction: column; }
.stat-value { font-size: 1.3rem; font-weight: 700; color: #1a1a2e; }
.stat-label { font-size: 0.78rem; color: #888; }

/* Drives */
.drive-row { display: flex; gap: 0.75rem; flex-wrap: wrap; }
.drive-bay { border: 2px solid #ddd; border-radius: 6px; padding: 0.6rem 0.8rem; min-width: 130px; flex: 1; max-width: 180px; }
.drive-bay.loaded { border-color: #27ae60; background: #f0faf4; }
.drive-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.3rem; }
.drive-id { font-weight: 600; font-size: 0.85rem; }
.drive-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.drive-dot.loaded { background: #27ae60; }
.drive-dot.empty { background: #bbb; }
.drive-serial { font-size: 0.75rem; color: #888; margin-bottom: 0.3rem; }
.drive-media { font-size: 0.8rem; }
.drive-barcode { font-weight: 600; display: block; color: #1a1a2e; }
.drive-pos { font-size: 0.72rem; color: #888; }
.drive-empty { font-size: 0.8rem; color: #bbb; font-style: italic; }

/* Slot Grid */
.slot-grid { display: grid; grid-template-columns: repeat(10, 1fr); gap: 4px; }
.slot-cell { border: 1px solid #e0e0e0; border-radius: 4px; padding: 0.3rem 0.25rem; text-align: center; min-height: 48px; display: flex; flex-direction: column; justify-content: center; font-size: 0.72rem; background: #fafafa; color: #bbb; }
.slot-cell.full { background: #e8f5e9; border-color: #a5d6a7; color: #1a1a2e; }
.slot-addr { font-size: 0.65rem; color: #aaa; }
.slot-cell.full .slot-addr { color: #888; }
.slot-barcode { font-weight: 600; font-size: 0.7rem; word-break: break-all; }

@media (max-width: 800px) {
  .slot-grid { grid-template-columns: repeat(5, 1fr); }
  .drive-bay { max-width: none; }
}
</style>
