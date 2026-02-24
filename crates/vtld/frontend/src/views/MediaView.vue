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
    error.value = 'Failed to fetch media data'
  }
}

useWebSocket(fetchData)
</script>

<template>
  <div class="media-view">
    <p v-if="error" class="error">{{ error }}</p>

    <template v-if="changer">
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
.media-view { max-width: 1400px; }
.error { color: #c0392b; margin-bottom: 1rem; }

.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }

.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
th { text-align: left; padding: 0.4rem 0.6rem; border-bottom: 2px solid #e0e0e0; color: #888; font-weight: 600; font-size: 0.78rem; text-transform: uppercase; }
td { padding: 0.35rem 0.6rem; border-bottom: 1px solid #f0f0f0; }
.mono { font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; font-size: 0.82rem; }
.row-full { background: #f0faf4; }
.row-except { background: #fff8e1; }

.barcode-link { text-decoration: none; font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; color: #1a1a2e; font-weight: 600; }
.barcode-link:hover { text-decoration: underline; color: #2980b9; }
</style>
