<script setup lang="ts">
import { onMounted } from 'vue'

onMounted(async () => {
  const mod = await import('swagger-ui-dist')
  const SwaggerUI = mod.SwaggerUIBundle as any
  SwaggerUI({
    url: '/api/openapi.json',
    dom_id: '#swagger-ui',
    presets: [SwaggerUI.presets.apis],
    layout: 'BaseLayout',
  })
})
</script>

<template>
  <div>
    <h2>API Documentation</h2>
    <div id="swagger-ui"></div>
  </div>
</template>

<style>
@import 'swagger-ui-dist/swagger-ui.css';
h2 { margin-bottom: 1rem; }
</style>
