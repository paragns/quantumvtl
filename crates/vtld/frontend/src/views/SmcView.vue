<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { apiFetch, fetchScsiLog, type ScsiLogSummary } from '../api'
import { useWebSocket } from '../composables/useWebSocket'
import ScsiLogLine from '../components/ScsiLogLine.vue'
import type { ChangerDetail } from '../types'

const changer = ref<ChangerDetail | null>(null)
const changerLog = ref<ScsiLogSummary[]>([])
const error = ref('')

async function fetchData() {
  try {
    const resp = await apiFetch('/api/vtl/changer')
    if (resp.ok) {
      changer.value = await resp.json()
      error.value = ''
    } else {
      error.value = `API error: ${resp.status}`
    }
  } catch {
    error.value = 'Failed to fetch changer data'
  }
  const cl = await fetchScsiLog('changer', 0, 4)
  if (cl) changerLog.value = cl.entries
}

useWebSocket(fetchData)

const nowMs = ref(Date.now())
let elapsedTimer: ReturnType<typeof setInterval> | null = null
let pendingRefetch = false

onMounted(() => {
  elapsedTimer = setInterval(() => {
    nowMs.value = Date.now()
    const state = changer.value?.state?.toLowerCase()
    if ((state === 'moving' || state === 'scanning') && !changer.value?.robot_operation) {
      if (!pendingRefetch) {
        pendingRefetch = true
        fetchData().finally(() => { pendingRefetch = false })
      }
    }
  }, 1000)
})

onUnmounted(() => {
  if (elapsedTimer) clearInterval(elapsedTimer)
})

const robotSnapTransition = ref(false)
let lastRobotStartMs = 0

watch(() => changer.value?.robot_operation?.started_at_ms, (ms) => {
  if (ms != null && ms !== lastRobotStartMs) {
    lastRobotStartMs = ms
    robotSnapTransition.value = true
    requestAnimationFrame(() => { robotSnapTransition.value = false })
  }
})

const robotProgressPct = computed(() => {
  const op = changer.value?.robot_operation
  if (!op) return 0
  const elapsed = (nowMs.value - op.started_at_ms) / 1000
  return Math.min(100, (elapsed / op.estimated_secs) * 100)
})

const robotElapsedSecs = computed(() => {
  const op = changer.value?.robot_operation
  if (!op) return 0
  return Math.max(0, (nowMs.value - op.started_at_ms) / 1000)
})

const robotDescription = computed(() => {
  const op = changer.value?.robot_operation
  if (!op) return ''
  if (op.kind === 'moving' && op.source != null && op.dest != null) {
    const src = '0x' + op.source.toString(16).padStart(4, '0').toUpperCase()
    const dst = '0x' + op.dest.toString(16).padStart(4, '0').toUpperCase()
    return `Moving medium ${src} \u2192 ${dst}`
  }
  if (op.kind === 'scanning') return 'Scanning inventory'
  return 'Robot operation in progress'
})
</script>

<template>
  <div class="smc-view">
    <p v-if="error" class="error">{{ error }}</p>

    <template v-if="changer">
      <!-- Identity -->
      <section class="card">
        <div class="identity-header">
          <div>
            <h3>{{ changer.vendor }} {{ changer.product }}</h3>
            <p class="meta">S/N: {{ changer.serial }} &middot; FW: {{ changer.firmware_version }}</p>
          </div>
          <span class="badge" :class="changer.state.toLowerCase()">{{ changer.state }}</span>
        </div>
      </section>

      <!-- Environmental -->
      <section class="card">
        <h3>Environmental</h3>
        <div class="stats">
          <div class="stat">
            <span class="stat-value">{{ changer.temperature_c }}&deg;C</span>
            <span class="stat-label">Temperature</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ changer.humidity_pct }}%</span>
            <span class="stat-label">Humidity</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ changer.total_moves }}</span>
            <span class="stat-label">Total Moves</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ changer.picker_position }}</span>
            <span class="stat-label">Picker Pos</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ changer.num_drives }}</span>
            <span class="stat-label">Drives</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ changer.num_slots }}</span>
            <span class="stat-label">Slots</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ changer.num_import_export }}</span>
            <span class="stat-label">I/E Ports</span>
          </div>
          <div class="stat" v-if="changer.prevent_medium_removal">
            <span class="stat-value warn">Locked</span>
            <span class="stat-label">Medium Removal</span>
          </div>
          <div v-if="changer.robot_operation" class="stat robot-stat">
            <div class="robot-inline">
              <span class="robot-inline-desc">{{ robotDescription }}</span>
              <div class="robot-inline-bar">
                <div class="robot-inline-fill" :class="{ 'no-transition': robotSnapTransition }" :style="{ width: robotProgressPct + '%' }"></div>
              </div>
              <span class="robot-inline-time" v-if="robotProgressPct < 100">{{ robotElapsedSecs.toFixed(1) }}s / {{ changer.robot_operation.estimated_secs.toFixed(1) }}s</span>
              <span class="robot-inline-time" v-else>{{ robotElapsedSecs.toFixed(1) }}s — taking longer than expected</span>
            </div>
          </div>
        </div>
      </section>

      <!-- Alerts -->
      <section class="card" v-if="changer.active_alerts.length">
        <h3>Active Alerts</h3>
        <div class="alert-list">
          <span v-for="a in changer.active_alerts" :key="a" class="alert-badge">TapeAlert #{{ a }}</span>
        </div>
      </section>

      <!-- SCSI Activity -->
      <section class="card">
        <h3><router-link to="/device/changer" class="card-title-link">SCSI Activity</router-link></h3>
        <div v-if="changerLog.length > 0">
          <ScsiLogLine
            v-for="e in changerLog"
            :key="e.seq"
            :entry="e"
            device-type="changer"
            :device-id="0"
          />
        </div>
        <p v-else class="no-activity">No recent activity</p>
      </section>

      <!-- Elements Table -->
      <section class="card">
        <h3>Elements ({{ changer.elements.length }})</h3>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Address</th>
                <th>Type</th>
                <th>Full</th>
                <th>Barcode</th>
                <th>Source</th>
                <th>Access</th>
                <th>Except</th>
                <th>Disabled</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="e in changer.elements" :key="e.address" :class="{ 'row-full': e.full, 'row-except': e.except }">
                <td class="mono">0x{{ e.address.toString(16).padStart(4, '0').toUpperCase() }}</td>
                <td>{{ e.element_type }}</td>
                <td>{{ e.full ? 'Yes' : '' }}</td>
                <td class="mono"><router-link v-if="e.barcode" :to="`/media/${e.barcode}`" class="barcode-link">{{ e.barcode }}</router-link></td>
                <td class="mono">{{ e.source_element ? '0x' + e.source_element.toString(16).padStart(4, '0').toUpperCase() : '' }}</td>
                <td>{{ e.access ? 'Yes' : '' }}</td>
                <td>{{ e.except ? 'Yes' : '' }}</td>
                <td>{{ e.disabled ? 'Yes' : '' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </template>
  </div>
</template>

<style scoped>
.smc-view { max-width: 1200px; }
.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }
.error { color: #c0392b; margin-bottom: 1rem; }

.identity-header { display: flex; justify-content: space-between; align-items: flex-start; }
.identity-header h3 { font-size: 1.1rem; margin-bottom: 0.15rem; }
.meta { color: #888; font-size: 0.85rem; }
.badge { display: inline-block; padding: 0.2rem 0.7rem; border-radius: 999px; font-size: 0.8rem; font-weight: 600; text-transform: capitalize; }
.badge.ready { background: #d4edda; color: #155724; }
.badge.initializing, .badge.scanning, .badge.moving { background: #fff3cd; color: #856404; }
.badge.notready { background: #f8d7da; color: #721c24; }

.stats { display: flex; gap: 2rem; flex-wrap: wrap; }
.stat { display: flex; flex-direction: column; }
.stat-value { font-size: 1.3rem; font-weight: 700; color: #1a1a2e; }
.stat-value.warn { color: #c0392b; }
.stat-label { font-size: 0.78rem; color: #888; }

.alert-list { display: flex; gap: 0.5rem; flex-wrap: wrap; }
.alert-badge { background: #f8d7da; color: #721c24; padding: 0.2rem 0.6rem; border-radius: 4px; font-size: 0.8rem; font-weight: 600; }

.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
th { text-align: left; padding: 0.4rem 0.6rem; border-bottom: 2px solid #e0e0e0; color: #888; font-weight: 600; font-size: 0.78rem; text-transform: uppercase; }
td { padding: 0.35rem 0.6rem; border-bottom: 1px solid #f0f0f0; }
.mono { font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; font-size: 0.82rem; }
.row-full { background: #f0faf4; }
.row-except { background: #fff8e1; }
.card-title-link { color: #1a1a2e; text-decoration: none; }
.card-title-link:hover { text-decoration: underline; }
.no-activity { color: #bbb; font-style: italic; font-size: 0.85rem; }
.barcode-link { color: #1a1a2e; text-decoration: none; font-weight: 600; }
.barcode-link:hover { text-decoration: underline; color: #2980b9; }

/* Inline Robot Progress */
.robot-stat { flex: 1 1 100%; }
.robot-inline { display: flex; align-items: center; gap: 0.6rem; }
.robot-inline-desc { font-size: 0.82rem; font-weight: 600; color: #1a1a2e; white-space: nowrap; }
.robot-inline-bar { flex: 1; height: 6px; background: #e0e0e0; border-radius: 3px; overflow: hidden; min-width: 80px; }
.robot-inline-fill { height: 100%; background: #2980b9; border-radius: 3px; transition: width 1s linear; }
.robot-inline-fill.no-transition { transition: none; }
.robot-inline-time { font-size: 0.75rem; color: #888; white-space: nowrap; }
</style>
