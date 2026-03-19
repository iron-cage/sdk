<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from 'vue-sonner'
import IconLogIn from '@/components/icons/IconLogIn.vue'

const router = useRouter()
const authStore = useAuthStore()

const email = ref('')
const password = ref('')
const loading = ref(false)
const loginError = ref('')

async function handleLogin() {
  loginError.value = ''
  loading.value = true

  try {
    await authStore.login({
      email: email.value,
      password: password.value,
    })
    router.push('/dashboard')
  } catch (err) {
    loginError.value = err instanceof Error ? err.message : 'Unexpected error'
    toast.error(loginError.value)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-background">
    <div class="w-full max-w-sm px-4">
      <h1 class="text-lg font-semibold text-foreground mb-1 text-center">Iron Cage</h1>
      <p class="text-base text-muted-foreground mb-6 text-center">Sign in to continue</p>
      <form class="space-y-4" @submit.prevent="handleLogin">
        <div class="space-y-2">
          <Label for="email">Email</Label>
          <Input
            id="email"
            v-model="email"
            type="email"
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

        <p v-if="loginError" role="alert" class="text-sm text-destructive text-center">{{ loginError }}</p>

        <Button type="submit" :disabled="loading" class="w-full">
          <IconLogIn />
          {{ loading ? 'Signing in...' : 'Sign in' }}
        </Button>
      </form>
    </div>
  </div>
</template>
