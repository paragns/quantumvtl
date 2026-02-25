<script setup lang="ts">
import { ref } from 'vue'
import type { DataField } from '../api'

const props = defineProps<{
  fields: DataField[]
  depth?: number
}>()

const currentDepth = props.depth ?? 0

const expanded = ref<Record<number, boolean>>({})

function toggle(index: number) {
  expanded.value[index] = !expanded.value[index]
}

function isExpanded(index: number): boolean {
  return !!expanded.value[index]
}

function hasChildren(field: DataField): boolean {
  return !!(field.children && field.children.length > 0)
}

function isFlag(field: DataField): boolean {
  return field.bit_range !== null && field.bit_range !== undefined
    && !field.bit_range.includes(':')
    && (field.hex_value === '0' || field.hex_value === '1')
}

function isFlagSet(field: DataField): boolean {
  return isFlag(field) && field.hex_value === '1'
}

function isReserved(field: DataField): boolean {
  const name = field.name.toLowerCase()
  return name === 'reserved' || name.startsWith('reserved')
}
</script>

<template>
  <template v-for="(field, i) in fields" :key="i">
    <tr
      :class="{
        'row-parent': hasChildren(field),
        'row-reserved': isReserved(field),
        'row-flag-set': isFlagSet(field),
        'row-flag-clear': isFlag(field) && !isFlagSet(field),
      }"
      @click="hasChildren(field) ? toggle(i) : undefined"
    >
      <td class="field-name" :style="{ paddingLeft: (currentDepth * 16 + 6) + 'px' }">
        <span v-if="hasChildren(field)" class="toggle-icon">{{ isExpanded(i) ? '\u25BC' : '\u25B6' }}</span>
        <span v-else class="toggle-spacer"></span>
        {{ field.name }}
      </td>
      <td class="mono">{{ field.byte_offset }}</td>
      <td class="mono">{{ field.bit_range ?? '-' }}</td>
      <td class="mono hex-val">{{ field.hex_value }}</td>
      <td class="decoded-val">{{ field.decoded }}</td>
    </tr>
    <DataFieldTree
      v-if="hasChildren(field) && isExpanded(i)"
      :fields="field.children!"
      :depth="currentDepth + 1"
    />
  </template>
</template>

<style scoped>
.field-name {
  font-weight: 600;
  color: #1a1a2e;
  white-space: nowrap;
  cursor: default;
}
.row-parent .field-name {
  cursor: pointer;
}
.toggle-icon {
  display: inline-block;
  width: 14px;
  font-size: 0.65rem;
  color: #888;
  text-align: center;
  margin-right: 2px;
}
.toggle-spacer {
  display: inline-block;
  width: 14px;
  margin-right: 2px;
}
.mono {
  font-family: 'SF Mono', 'Consolas', 'Liberation Mono', monospace;
}
.hex-val {
  color: #555;
}
.decoded-val {
  color: #333;
}
.row-reserved .field-name,
.row-reserved .decoded-val {
  color: #bbb;
}
.row-reserved .hex-val {
  color: #ccc;
}
.row-flag-set .decoded-val {
  color: #27ae60;
  font-weight: 600;
}
.row-flag-clear .decoded-val {
  color: #999;
}
.row-parent > td {
  background: #fafbfc;
}
tr {
  border-bottom: 1px solid #f0f0f0;
}
td {
  padding: 0.25rem 0.4rem;
  font-size: 0.78rem;
}
</style>
