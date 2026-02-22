<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()
const username = ref('')
const password = ref('')
const error = ref('')

async function doLogin() {
  error.value = ''
  try {
    const resp = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: username.value, password: password.value }),
    })
    if (!resp.ok) {
      const body = await resp.json()
      error.value = body.error || 'Login failed'
      return
    }
    const data = await resp.json()
    localStorage.setItem('token', data.token)
    const redirect = (route.query.redirect as string) || '/'
    router.push(redirect)
  } catch (e) {
    error.value = 'Network error'
  }
}
</script>

<template>
  <div class="login-page">
    <div class="login-card">
      <h1>QuantumVTL</h1>
      <form @submit.prevent="doLogin">
        <div class="field">
          <label>Username</label>
          <input v-model="username" type="text" autocomplete="username" />
        </div>
        <div class="field">
          <label>Password</label>
          <input v-model="password" type="password" autocomplete="current-password" />
        </div>
        <p v-if="error" class="error">{{ error }}</p>
        <button type="submit">Login</button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.login-page { display: flex; justify-content: center; align-items: center; min-height: 80vh; }
.login-card { background: #fff; border-radius: 8px; padding: 2rem; box-shadow: 0 2px 8px rgba(0,0,0,0.1); width: 360px; }
.login-card h1 { text-align: center; margin-bottom: 1.5rem; color: #1a1a2e; }
.field { margin-bottom: 1rem; }
.field label { display: block; margin-bottom: 0.25rem; font-size: 0.9rem; color: #555; }
.field input { width: 100%; padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px; font-size: 1rem; }
button { width: 100%; padding: 0.6rem; background: #1a1a2e; color: #fff; border: none; border-radius: 4px; font-size: 1rem; cursor: pointer; }
button:hover { background: #16213e; }
.error { color: #c0392b; font-size: 0.85rem; margin-bottom: 0.5rem; }
</style>
