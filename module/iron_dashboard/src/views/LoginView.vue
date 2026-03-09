<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Alert, AlertDescription } from '@/components/ui/alert'

const router = useRouter()
const authStore = useAuthStore()

const email = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

async function handleLogin() {
  error.value = ''
  loading.value = true

  try {
    await authStore.login({
      email: email.value,
      password: password.value,
    })
    router.push('/dashboard')
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Unexpected error"
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-background">
    <div class="w-full max-w-sm px-4">
      <h1 class="text-lg font-semibold text-foreground mb-1">Iron Cage</h1>
      <p class="text-base text-muted-foreground mb-6">Sign in to continue</p>
      <form class="space-y-4" @submit.prevent="handleLogin">
        <Alert v-if="error" variant="destructive">
          <AlertDescription>{{ error }}</AlertDescription>
        </Alert>

        <div class="space-y-2">
          <Label for="email">Email</Label>
          <Input
            id="email"
            v-model="email"
            type="text"
            required
            :disabled="loading"
          />
        </div>

        <div class="space-y-2">
          <Label for="password">Password</Label>
          <Input
            id="password"
            v-model="password"
            type="password"
            required
            :disabled="loading"
          />
        </div>

        <Button type="submit" :disabled="loading" class="w-full">
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" /></svg>
          {{ loading ? 'Signing in...' : 'Sign in' }}
        </Button>
      </form>
    </div>
  </div>
</template>
