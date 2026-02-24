<script setup lang="ts">
import { ref } from 'vue'
import { apiFetch } from '../api'
import { useWebSocket } from '../composables/useWebSocket'

interface ConfigEntry {
  key: string
  value: string
}

const entries = ref<ConfigEntry[]>([])
const error = ref('')

async function fetchData() {
  try {
    const resp = await apiFetch('/api/config')
    if (resp.ok) {
      entries.value = await resp.json()
      error.value = ''
    } else {
      error.value = `API error: ${resp.status}`
    }
  } catch {
    error.value = 'Failed to fetch configuration'
  }
}

useWebSocket(fetchData)
</script>

<template>
  <div class="config-view">
    <p v-if="error" class="error">{{ error }}</p>

    <section class="card">
      <h3>Configuration</h3>
      <div v-if="entries.length > 0" class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Key</th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="e in entries" :key="e.key">
              <td class="mono">{{ e.key }}</td>
              <td class="mono">{{ e.value }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-else class="empty-state">No configuration entries</p>
    </section>
  </div>
</template>

<style scoped>
.config-view { max-width: 1200px; }
.error { color: #c0392b; margin-bottom: 1rem; }

.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }

.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
th { text-align: left; padding: 0.4rem 0.6rem; border-bottom: 2px solid #e0e0e0; color: #888; font-weight: 600; font-size: 0.78rem; text-transform: uppercase; }
td { padding: 0.4rem 0.6rem; border-bottom: 1px solid #f0f0f0; }
.mono { font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; font-size: 0.82rem; }

.empty-state { color: #888; font-style: italic; font-size: 0.9rem; }
</style>
