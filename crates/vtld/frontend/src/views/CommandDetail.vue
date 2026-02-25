<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { fetchScsiLogEntry, type ScsiCommandDetail } from '../api'
import DataFieldTree from '../components/DataFieldTree.vue'

const route = useRoute()

const deviceType = computed(() => {
  return route.path.startsWith('/device/changer') ? 'changer' : 'drive'
})
const deviceId = computed(() => {
  if (deviceType.value === 'changer') return 0
  return Number(route.params.id) || 0
})
const seq = computed(() => Number(route.params.seq) || 0)
const deviceLabel = computed(() => {
  if (deviceType.value === 'changer') return 'Media Changer'
  return `Tape Drive ${deviceId.value}`
})
const devicePath = computed(() => {
  if (deviceType.value === 'changer') return '/device/changer'
  return `/device/drive/${deviceId.value}`
})

const detail = ref<ScsiCommandDetail | null>(null)
const error = ref('')
const showRawDataIn = ref(false)
const showRawSense = ref(false)
const nowMs = ref(Date.now())
let elapsedTimer: ReturnType<typeof setInterval> | null = null
let ws: WebSocket | null = null
let destroyed = false

const isCompleted = computed(() => detail.value?.completed ?? true)

const elapsedText = computed(() => {
  if (!detail.value) return ''
  const startMs = new Date(detail.value.timestamp).getTime()
  const elapsed = Math.max(0, nowMs.value - startMs) / 1000
  return `${elapsed.toFixed(1)}s elapsed`
})

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

const hasStructuredDataIn = computed(() => {
  const rb = detail.value?.response_breakdown
  return rb?.data_in_fields && rb.data_in_fields.length > 0
})

const hasStructuredSense = computed(() => {
  const sense = detail.value?.response_breakdown?.sense
  return sense?.fields && sense.fields.length > 0
})

async function loadDetail() {
  const data = await fetchScsiLogEntry(deviceType.value, deviceId.value, seq.value)
  if (data) {
    detail.value = data
    if (data.completed && elapsedTimer) {
      clearInterval(elapsedTimer)
      elapsedTimer = null
    }
  }
}

function connectWebSocket() {
  if (destroyed) return
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
  ws = new WebSocket(`${proto}//${location.host}/api/ws`)
  ws.onmessage = () => { loadDetail() }
  ws.onclose = () => {
    ws = null
    if (!destroyed) {
      setTimeout(() => { connectWebSocket() }, 5000)
    }
  }
}

onMounted(() => {
  loadDetail()
  connectWebSocket()
  elapsedTimer = setInterval(() => { nowMs.value = Date.now() }, 1000)
})

onUnmounted(() => {
  destroyed = true
  if (elapsedTimer) clearInterval(elapsedTimer)
  ws?.close()
})
</script>

<template>
  <div class="command-detail">
    <nav class="breadcrumb">
      <router-link to="/">Dashboard</router-link>
      <span class="sep">/</span>
      <router-link :to="devicePath">{{ deviceLabel }}</router-link>
      <span class="sep">/</span>
      <span>Command #{{ seq }}</span>
    </nav>

    <p v-if="error" class="error">{{ error }}</p>

    <template v-if="detail">
      <div class="top-bar">
        <h2>{{ detail.opcode_name }}</h2>
        <div class="meta">
          <span class="meta-item">{{ formatTime(detail.timestamp) }}</span>
          <span v-if="isCompleted" class="meta-item">{{ formatDuration(detail.duration_us) }}</span>
          <span v-else class="meta-item in-progress-dur">{{ elapsedText }}</span>
          <span v-if="!isCompleted" class="status-badge in-progress">IN PROGRESS</span>
          <span v-else class="status-badge" :class="{ good: detail.status === 0, error: detail.status !== 0 }">
            {{ detail.status_name }}
          </span>
        </div>
      </div>

      <div class="panels">
        <!-- Request Panel -->
        <section class="card panel">
          <h3>Request</h3>

          <div class="section-label">CDB Fields</div>
          <table class="field-table">
            <thead>
              <tr>
                <th>Field</th>
                <th>Byte</th>
                <th>Bits</th>
                <th>Hex</th>
                <th>Decoded</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(f, i) in detail.cdb_breakdown.fields" :key="i">
                <td class="field-name">{{ f.name }}</td>
                <td class="mono">{{ f.byte_offset }}</td>
                <td class="mono">{{ f.bit_range ?? '-' }}</td>
                <td class="mono">{{ f.hex_value }}</td>
                <td>{{ f.decoded }}</td>
              </tr>
            </tbody>
          </table>

          <div class="section-label">CDB Hex Dump</div>
          <pre class="hex-dump">{{ detail.cdb_hex }}</pre>

          <template v-if="detail.data_out_len > 0">
            <div class="section-label">Data-Out</div>
            <pre v-if="detail.data_out_hex" class="hex-dump">{{ detail.data_out_hex }}</pre>
            <p v-else class="omitted">Payload omitted ({{ detail.data_out_len }} bytes)</p>
          </template>
        </section>

        <!-- Response Panel -->
        <section class="card panel">
          <h3>Response</h3>

          <div class="response-status">
            <span v-if="!isCompleted" class="status-badge large in-progress">IN PROGRESS</span>
            <span v-else class="status-badge large" :class="{ good: detail.status === 0, error: detail.status !== 0 }">
              {{ detail.status_name }}
            </span>
            <span v-if="isCompleted" class="duration-note">Command took {{ formatDuration(detail.duration_us) }}</span>
            <span v-else class="duration-note in-progress-dur">{{ elapsedText }}</span>
          </div>

          <!-- Sense Data -->
          <template v-if="detail.response_breakdown.sense">
            <div class="section-header">
              <div class="section-label">Sense Data</div>
              <button
                v-if="hasStructuredSense"
                class="toggle-btn"
                @click="showRawSense = !showRawSense"
              >{{ showRawSense ? 'Structured' : 'Raw Hex' }}</button>
            </div>

            <!-- Sense summary (always shown) -->
            <div class="sense-grid">
              <div class="sense-item">
                <span class="sense-label">Sense Key</span>
                <span class="sense-value error">{{ detail.response_breakdown.sense.sense_key_name }} (0x{{ detail.response_breakdown.sense.sense_key.toString(16).toUpperCase().padStart(2, '0') }})</span>
              </div>
              <div class="sense-item">
                <span class="sense-label">ASC/ASCQ</span>
                <span class="sense-value">0x{{ detail.response_breakdown.sense.asc.toString(16).toUpperCase().padStart(2, '0') }} / 0x{{ detail.response_breakdown.sense.ascq.toString(16).toUpperCase().padStart(2, '0') }}</span>
              </div>
              <div class="sense-item full">
                <span class="sense-label">Description</span>
                <span class="sense-value">{{ detail.response_breakdown.sense.asc_description }}</span>
              </div>
            </div>

            <!-- Structured sense fields -->
            <template v-if="hasStructuredSense && !showRawSense">
              <table class="field-table">
                <thead>
                  <tr>
                    <th>Field</th>
                    <th>Byte</th>
                    <th>Bits</th>
                    <th>Hex</th>
                    <th>Decoded</th>
                  </tr>
                </thead>
                <tbody>
                  <DataFieldTree :fields="detail.response_breakdown.sense.fields!" />
                </tbody>
              </table>
            </template>

            <!-- Raw sense hex -->
            <pre v-if="!hasStructuredSense || showRawSense" class="hex-dump">{{ detail.response_breakdown.sense.hex_dump }}</pre>
          </template>

          <!-- Data-In Response -->
          <template v-if="detail.data_in_len > 0">
            <div class="section-header">
              <div class="section-label">Data-In ({{ detail.data_in_len }} bytes)</div>
              <button
                v-if="hasStructuredDataIn && detail.data_in_hex"
                class="toggle-btn"
                @click="showRawDataIn = !showRawDataIn"
              >{{ showRawDataIn ? 'Structured' : 'Raw Hex' }}</button>
            </div>

            <!-- Structured data-in fields -->
            <template v-if="hasStructuredDataIn && !showRawDataIn">
              <table class="field-table">
                <thead>
                  <tr>
                    <th>Field</th>
                    <th>Byte</th>
                    <th>Bits</th>
                    <th>Hex</th>
                    <th>Decoded</th>
                  </tr>
                </thead>
                <tbody>
                  <DataFieldTree :fields="detail.response_breakdown.data_in_fields!" />
                </tbody>
              </table>
            </template>

            <!-- Raw hex (fallback or toggled) -->
            <template v-if="!hasStructuredDataIn || showRawDataIn">
              <pre v-if="detail.data_in_hex" class="hex-dump">{{ detail.data_in_hex }}</pre>
              <p v-else class="omitted">Payload omitted ({{ detail.data_in_len }} bytes)</p>
            </template>
          </template>

          <template v-if="detail.sense_hex && !detail.response_breakdown.sense">
            <div class="section-label">Raw Sense</div>
            <pre class="hex-dump">{{ detail.sense_hex }}</pre>
          </template>
        </section>
      </div>
    </template>
  </div>
</template>

<style scoped>
.command-detail { max-width: 1200px; margin: 0 auto; }
.breadcrumb { font-size: 0.85rem; margin-bottom: 0.75rem; }
.breadcrumb a { color: #555; text-decoration: none; }
.breadcrumb a:hover { color: #1a1a2e; text-decoration: underline; }
.sep { margin: 0 0.4rem; color: #aaa; }
.error { color: #c0392b; margin-bottom: 1rem; }

.top-bar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem; }
.top-bar h2 { margin: 0; }
.meta { display: flex; align-items: center; gap: 0.75rem; }
.meta-item { font-size: 0.85rem; color: #666; }

.panels { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }

.section-header { display: flex; justify-content: space-between; align-items: center; }
.section-label { font-size: 0.75rem; font-weight: 600; text-transform: uppercase; color: #888; margin: 0.75rem 0 0.35rem; }
.toggle-btn {
  font-size: 0.7rem;
  padding: 0.15rem 0.5rem;
  border: 1px solid #ddd;
  border-radius: 3px;
  background: #f8f9fa;
  color: #555;
  cursor: pointer;
  margin-top: 0.5rem;
}
.toggle-btn:hover { background: #e9ecef; border-color: #ccc; }

.hex-dump { background: #f5f5f5; padding: 0.5rem 0.75rem; border-radius: 4px; font-family: 'SF Mono', 'Consolas', monospace; font-size: 0.78rem; word-break: break-all; white-space: pre-wrap; color: #333; margin: 0; }
.omitted { font-size: 0.82rem; color: #888; font-style: italic; }

.field-table { width: 100%; border-collapse: collapse; font-size: 0.78rem; }
.field-table th { text-align: left; padding: 0.3rem 0.4rem; border-bottom: 1px solid #e0e0e0; font-size: 0.7rem; color: #888; text-transform: uppercase; }
.field-table td { padding: 0.25rem 0.4rem; border-bottom: 1px solid #f0f0f0; }
.field-name { font-weight: 600; color: #1a1a2e; }
.mono { font-family: 'SF Mono', 'Consolas', monospace; }

.response-status { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; }
.status-badge { display: inline-block; padding: 0.15rem 0.5rem; border-radius: 3px; font-size: 0.78rem; font-weight: 600; }
.status-badge.large { font-size: 0.9rem; padding: 0.2rem 0.6rem; }
.status-badge.good { background: #d4edda; color: #155724; }
.status-badge.error { background: #f8d7da; color: #721c24; }
.status-badge.in-progress { background: #fff3cd; color: #856404; }
.in-progress-dur { color: #e67e22; animation: pulse 1.2s ease-in-out infinite; }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.3; } }
.duration-note { font-size: 0.82rem; color: #666; }

.sense-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; margin-bottom: 0.5rem; }
.sense-item { display: flex; flex-direction: column; }
.sense-item.full { grid-column: 1 / -1; }
.sense-label { font-size: 0.7rem; color: #888; }
.sense-value { font-size: 0.85rem; font-weight: 600; }
.sense-value.error { color: #c0392b; }

@media (max-width: 800px) {
  .panels { grid-template-columns: 1fr; }
}
</style>
