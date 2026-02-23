<script setup lang="ts">
import { ref } from 'vue'
import { apiFetch } from '../api'
import { useWebSocket } from '../composables/useWebSocket'
import type { ChangerDetail } from '../types'

const changer = ref<ChangerDetail | null>(null)
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
}

useWebSocket(fetchData)
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
        </div>
      </section>

      <!-- Alerts -->
      <section class="card" v-if="changer.active_alerts.length">
        <h3>Active Alerts</h3>
        <div class="alert-list">
          <span v-for="a in changer.active_alerts" :key="a" class="alert-badge">TapeAlert #{{ a }}</span>
        </div>
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
                <td class="mono">{{ e.barcode ?? '' }}</td>
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
.badge.initializing, .badge.scanning { background: #fff3cd; color: #856404; }
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
</style>
