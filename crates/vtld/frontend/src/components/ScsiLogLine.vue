<script setup lang="ts">
import type { ScsiLogSummary } from '../api'

const props = defineProps<{
  entry: ScsiLogSummary
  deviceType: string
  deviceId: number
}>()

function formatTime(ts: string): string {
  try {
    const d = new Date(ts)
    const hh = String(d.getHours()).padStart(2, '0')
    const mm = String(d.getMinutes()).padStart(2, '0')
    const ss = String(d.getSeconds()).padStart(2, '0')
    const ms = String(d.getMilliseconds()).padStart(3, '0')
    return `${hh}:${mm}:${ss}.${ms}`
  } catch {
    return ts
  }
}

function formatDuration(us: number): string {
  if (us < 1000) return `${us}us`
  if (us < 1000000) return `${(us / 1000).toFixed(1)}ms`
  return `${(us / 1000000).toFixed(2)}s`
}

const cmdPath = props.deviceType === 'changer'
  ? `/device/changer/cmd/${props.entry.seq}`
  : `/device/drive/${props.deviceId}/cmd/${props.entry.seq}`
</script>

<template>
  <router-link :to="cmdPath" class="scsi-log-line">
    <span class="ts">{{ formatTime(entry.timestamp) }}</span>
    <span class="op">{{ entry.opcode_name }}</span>
    <span class="arrow">-&gt;</span>
    <span class="status" :class="{ good: entry.status === 0, error: entry.status !== 0 }">{{ entry.status_name }}</span>
    <span class="dur">{{ formatDuration(entry.duration_us) }}</span>
  </router-link>
</template>

<style scoped>
.scsi-log-line {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-family: 'SF Mono', 'Consolas', 'Liberation Mono', monospace;
  font-size: 0.78rem;
  padding: 0.2rem 0;
  text-decoration: none;
  color: inherit;
  border-bottom: 1px solid #f0f0f0;
}
.scsi-log-line:last-child { border-bottom: none; }
.scsi-log-line:hover { background: #f8f9fa; }
.ts { color: #888; white-space: nowrap; }
.op { font-weight: 600; min-width: 160px; color: #1a1a2e; }
.arrow { color: #aaa; }
.status.good { color: #27ae60; font-weight: 600; }
.status.error { color: #c0392b; font-weight: 600; }
.dur { color: #888; margin-left: auto; white-space: nowrap; }
</style>
