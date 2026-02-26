<script setup lang="ts">
import { ref } from 'vue'
import { apiFetch } from '../api'
import { useWebSocket } from '../composables/useWebSocket'

interface ConfigSetting {
  key: string
  description: string
  value: any
  default_value?: any
  current_value?: any
  required: boolean
}

interface ConfigSection {
  name: string
  description: string
  settings: ConfigSetting[]
}

interface FullConfigResponse {
  sections: ConfigSection[]
}

interface ConfigEntry {
  key: string
  value: string
}

const sections = ref<ConfigSection[]>([])
const kvEntries = ref<ConfigEntry[]>([])
const error = ref('')

async function fetchData() {
  try {
    const [fullResp, kvResp] = await Promise.all([
      apiFetch('/api/config/full'),
      apiFetch('/api/config'),
    ])
    if (fullResp.ok) {
      const data: FullConfigResponse = await fullResp.json()
      sections.value = data.sections
    } else {
      error.value = `API error: ${fullResp.status}`
    }
    if (kvResp.ok) {
      kvEntries.value = await kvResp.json()
    }
    if (fullResp.ok) error.value = ''
  } catch {
    error.value = 'Failed to fetch configuration'
  }
}

function formatValue(val: any): string {
  if (val === null || val === undefined) return '—'
  if (typeof val === 'object') {
    if ('count' in val && 'barcodes' in val) {
      return `${val.count} cartridge${val.count !== 1 ? 's' : ''}`
    }
    if ('count' in val) {
      return `${val.count} cartridge${val.count !== 1 ? 's' : ''}`
    }
    return JSON.stringify(val)
  }
  return String(val)
}

function isDefault(setting: ConfigSetting): boolean {
  if (setting.default_value === undefined || setting.default_value === null) return false
  return JSON.stringify(setting.value) === JSON.stringify(setting.default_value)
}

function hasRuntimeOverride(setting: ConfigSetting): boolean {
  return setting.current_value !== undefined && setting.current_value !== null
}

function getBarcodes(setting: ConfigSetting): string[] {
  if (typeof setting.value === 'object' && setting.value && 'barcodes' in setting.value) {
    return setting.value.barcodes
  }
  return []
}

useWebSocket(fetchData)
</script>

<template>
  <div class="config-view">
    <p v-if="error" class="error">{{ error }}</p>

    <section v-for="section in sections" :key="section.name" class="card">
      <h3>{{ section.name }}</h3>
      <p class="section-desc">{{ section.description }}</p>

      <div class="sub-card">
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Setting</th>
                <th>Description</th>
                <th>Value</th>
                <th>Default</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="s in section.settings" :key="s.key">
                <td class="mono key-cell">
                  {{ s.key }}
                  <span v-if="s.required" class="badge required">required</span>
                </td>
                <td class="desc-cell">{{ s.description }}</td>
                <td class="mono value-cell">
                  <template v-if="hasRuntimeOverride(s)">
                    <span class="runtime-value">{{ formatValue(s.current_value) }}</span>
                    <span class="badge runtime">runtime</span>
                    <div class="config-original">config: {{ formatValue(s.value) }}</div>
                  </template>
                  <template v-else>
                    <span>{{ formatValue(s.value) }}</span>
                    <span v-if="isDefault(s)" class="badge default">default</span>
                  </template>
                  <!-- Media barcodes list -->
                  <ul v-if="getBarcodes(s).length > 0" class="barcode-list">
                    <li v-for="bc in getBarcodes(s)" :key="bc">{{ bc }}</li>
                  </ul>
                </td>
                <td class="mono default-cell">
                  <template v-if="s.default_value !== undefined && s.default_value !== null">
                    {{ formatValue(s.default_value) }}
                  </template>
                  <template v-else>
                    <span class="no-default">—</span>
                  </template>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>

    <!-- KV store entries (original config endpoint) -->
    <section v-if="kvEntries.length > 0" class="card">
      <h3>Runtime Store</h3>
      <p class="section-desc">Key-value entries from the runtime database</p>
      <div class="sub-card">
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Key</th>
                <th>Value</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="e in kvEntries" :key="e.key">
                <td class="mono">{{ e.key }}</td>
                <td class="mono">{{ e.value }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.config-view { max-width: 1200px; }
.error { color: #c0392b; margin-bottom: 1rem; }

/* ===== Cards ===== */
.card {
  background: #fff;
  border-radius: 8px;
  padding: 1rem 1.25rem;
  margin-bottom: 1rem;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}
.card h3 { margin-bottom: 0.25rem; font-size: 1rem; color: #1a1a2e; }
.section-desc { color: #888; font-size: 0.82rem; margin-bottom: 0.75rem; }

/* Sub-cards */
.sub-card {
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  padding: 0.85rem 1rem;
  margin-top: 0.5rem;
}

/* ===== Tables ===== */
.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
th {
  text-align: left;
  padding: 0.4rem 0.6rem;
  border-bottom: 2px solid #e0e0e0;
  color: #888;
  font-weight: 600;
  font-size: 0.78rem;
  text-transform: uppercase;
}
td { padding: 0.4rem 0.6rem; border-bottom: 1px solid #f0f0f0; vertical-align: top; }
.mono { font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; font-size: 0.82rem; }

.key-cell { white-space: nowrap; }
.desc-cell { color: #555; min-width: 200px; }
.value-cell { min-width: 120px; }
.default-cell { color: #999; min-width: 80px; }
.no-default { color: #ccc; }

/* ===== Badges ===== */
.badge {
  display: inline-block;
  padding: 0.1rem 0.45rem;
  border-radius: 999px;
  font-size: 0.68rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.02em;
  margin-left: 0.4rem;
  vertical-align: middle;
}
.badge.required { background: #fff3cd; color: #856404; }
.badge.default { background: #d4edda; color: #155724; }
.badge.runtime { background: #cce5ff; color: #004085; }

/* Runtime override */
.runtime-value { font-weight: 600; color: #004085; }
.config-original { font-size: 0.75rem; color: #999; margin-top: 0.2rem; }

/* Barcode list */
.barcode-list {
  list-style: none;
  padding: 0;
  margin: 0.3rem 0 0 0;
}
.barcode-list li {
  font-size: 0.78rem;
  color: #555;
  padding: 0.1rem 0;
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
}
.barcode-list li::before {
  content: '▸ ';
  color: #aaa;
}

.empty-state { color: #888; font-style: italic; font-size: 0.9rem; }
</style>
