<script setup lang="ts">
import { ref, watch } from 'vue'
import { fetchMediaDetail, type MediaDetailResponse } from '../api'
import { useWebSocket } from '../composables/useWebSocket'

const props = defineProps<{ barcode: string }>()

const media = ref<MediaDetailResponse | null>(null)
const error = ref('')

async function fetchData() {
  try {
    const data = await fetchMediaDetail(props.barcode)
    if (data) {
      media.value = data
      error.value = ''
    } else {
      error.value = `Media "${props.barcode}" not found`
    }
  } catch {
    error.value = 'Failed to fetch media data'
  }
}

watch(() => props.barcode, () => fetchData())
useWebSocket(fetchData)

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
}
</script>

<template>
  <div class="media-detail">
    <router-link to="/smc" class="back-link">&larr; Library</router-link>

    <p v-if="error" class="error">{{ error }}</p>

    <template v-if="media">
      <!-- Identity -->
      <section class="card header-card">
        <div class="header-row">
          <div>
            <h3 class="barcode-title">{{ media.barcode }}</h3>
            <p class="meta">{{ media.medium_type }}</p>
          </div>
          <div class="badges">
            <span class="badge gen">{{ media.generation }}</span>
            <span v-if="media.write_protected" class="badge wp">WP</span>
            <span v-if="media.worm" class="badge worm">WORM</span>
          </div>
        </div>
      </section>

      <!-- Location -->
      <section class="card">
        <h3>Location</h3>
        <div class="stats">
          <div class="stat">
            <span class="stat-value">{{ media.location }}</span>
            <span class="stat-label">Current Position</span>
          </div>
          <div class="stat" v-if="media.in_drive != null">
            <router-link :to="`/ssc/drive/${media.in_drive}`" class="stat-value link">Drive {{ media.in_drive }}</router-link>
            <span class="stat-label">Loaded In</span>
          </div>
        </div>
      </section>

      <!-- Capacity -->
      <section class="card">
        <h3>Capacity</h3>
        <div class="stats">
          <div class="stat">
            <span class="stat-value">{{ formatBytes(media.native_bytes_written) }}</span>
            <span class="stat-label">Native Written</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ formatBytes(media.compressed_bytes_written) }}</span>
            <span class="stat-label">Compressed Written</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ media.approximate_remaining_mb.toLocaleString() }} MB</span>
            <span class="stat-label">Remaining</span>
          </div>
          <div class="stat" v-if="media.compression_enabled">
            <span class="stat-value">{{ media.compression_ratio.toFixed(2) }}:1</span>
            <span class="stat-label">Compression</span>
          </div>
        </div>
        <div class="progress-wrap" v-if="media.native_capacity_bytes > 0">
          <div class="progress-bar">
            <div class="progress-fill capacity" :style="{ width: media.capacity_used_pct + '%' }"></div>
          </div>
          <span class="progress-label">{{ media.capacity_used_pct.toFixed(1) }}% used of {{ formatBytes(media.native_capacity_bytes) }}</span>
        </div>
      </section>

      <!-- Lifetime -->
      <section class="card">
        <h3>Lifetime</h3>
        <div class="stats">
          <div class="stat">
            <span class="stat-value">{{ media.total_loads }}</span>
            <span class="stat-label">Total Loads</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ media.total_records.toLocaleString() }}</span>
            <span class="stat-label">Total Records</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ media.total_filemarks }}</span>
            <span class="stat-label">Total Filemarks</span>
          </div>
          <div class="stat">
            <span class="stat-value" :class="{ flag: !media.optimization_done }">{{ media.optimization_done ? 'Done' : 'Pending' }}</span>
            <span class="stat-label">Optimization</span>
          </div>
        </div>
      </section>

      <!-- Partitions -->
      <section class="card" v-if="media.partitions.length > 0">
        <h3>Partitions ({{ media.partition_count }})</h3>
        <div v-for="p in media.partitions" :key="p.index" class="partition-section">
          <h4>Partition {{ p.index }}</h4>
          <div class="stats">
            <div class="stat">
              <span class="stat-value">{{ p.record_count.toLocaleString() }}</span>
              <span class="stat-label">Records</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ p.filemark_count }}</span>
              <span class="stat-label">Filemarks</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ formatBytes(p.bytes_written_native) }}</span>
              <span class="stat-label">Native Written</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ formatBytes(p.bytes_written_compressed) }}</span>
              <span class="stat-label">Compressed Written</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ formatBytes(p.bytes_read_native) }}</span>
              <span class="stat-label">Native Read</span>
            </div>
          </div>
          <div v-if="p.filemark_positions.length > 0" class="filemark-table-wrap">
            <table class="filemark-table">
              <thead>
                <tr>
                  <th>#</th>
                  <th>Record Index</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(pos, i) in p.filemark_positions" :key="i">
                  <td class="mono">{{ i }}</td>
                  <td class="mono">{{ pos }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <!-- No data -->
      <section class="card" v-if="media.partition_count === 0">
        <p class="empty-state">This media has never been loaded into a drive. No tape data available.</p>
      </section>
    </template>
  </div>
</template>

<style scoped>
.media-detail { max-width: 900px; }
.back-link { display: inline-block; margin-bottom: 1rem; color: #1a1a2e; text-decoration: none; font-size: 0.9rem; }
.back-link:hover { text-decoration: underline; }
.error { color: #c0392b; margin-bottom: 1rem; }
.empty-state { color: #888; font-style: italic; }

.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }

.header-card .header-row { display: flex; justify-content: space-between; align-items: flex-start; }
.barcode-title { font-size: 1.3rem; font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; margin-bottom: 0.15rem; }
.meta { color: #888; font-size: 0.85rem; }
.badges { display: flex; gap: 0.4rem; }
.badge { display: inline-block; padding: 0.2rem 0.7rem; border-radius: 999px; font-size: 0.8rem; font-weight: 600; }
.badge.gen { background: #cce5ff; color: #004085; }
.badge.wp { background: #f8d7da; color: #721c24; }
.badge.worm { background: #d1ecf1; color: #0c5460; }

.stats { display: flex; gap: 2rem; flex-wrap: wrap; }
.stat { display: flex; flex-direction: column; }
.stat-value { font-size: 1.15rem; font-weight: 700; color: #1a1a2e; }
.stat-value.flag { color: #e67e22; }
.stat-value.link { color: #2980b9; text-decoration: none; }
.stat-value.link:hover { text-decoration: underline; }
.stat-label { font-size: 0.78rem; color: #888; }

.progress-wrap { margin-top: 0.75rem; }
.progress-bar { height: 8px; background: #e0e0e0; border-radius: 4px; overflow: hidden; }
.progress-fill { height: 100%; background: #1a1a2e; border-radius: 4px; transition: width 0.3s; }
.progress-fill.capacity { background: #27ae60; }
.progress-label { font-size: 0.78rem; color: #888; margin-top: 0.25rem; display: block; }

.partition-section { margin-bottom: 1.25rem; padding-bottom: 1rem; border-bottom: 1px solid #f0f0f0; }
.partition-section:last-child { border-bottom: none; margin-bottom: 0; padding-bottom: 0; }
.partition-section h4 { font-size: 0.9rem; font-weight: 600; color: #1a1a2e; margin-bottom: 0.5rem; }

.filemark-table-wrap { margin-top: 0.5rem; overflow-x: auto; }
.filemark-table { width: auto; border-collapse: collapse; font-size: 0.82rem; }
.filemark-table th { text-align: left; padding: 0.3rem 0.8rem; border-bottom: 2px solid #e0e0e0; color: #888; font-weight: 600; font-size: 0.75rem; text-transform: uppercase; }
.filemark-table td { padding: 0.25rem 0.8rem; border-bottom: 1px solid #f0f0f0; }
.mono { font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; }
</style>
