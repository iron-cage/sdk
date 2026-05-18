<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Check, Copy } from 'lucide-vue-next'
import { useApi, type MeWorkspaceResponse, type CreateTokenResponse } from '@/composables/useApi'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'

const api = useApi()
const authStore = useAuthStore()

const gatewayUrl = computed(() => import.meta.env.VITE_GATEWAY_URL || window.location.origin)

const workspaceLoading = ref(true)
const workspaceError = ref<string | null>(null)
const workspace = ref<MeWorkspaceResponse | null>(null)

const generating = ref(false)
const generateError = ref<string | null>(null)
const icToken = ref<CreateTokenResponse | null>(null)

const copiedGateway = ref(false)
const copiedToken = ref(false)

onMounted(async () => {
  try {
    workspace.value = await api.getMeWorkspace()
  } catch (e) {
    workspaceError.value = e instanceof Error ? e.message : String(e)
  } finally {
    workspaceLoading.value = false
  }
})

async function generateIcToken() {
  if (!authStore.username) {
    generateError.value = 'Not authenticated'
    return
  }
  generating.value = true
  generateError.value = null
  try {
    icToken.value = await api.createToken({
      user_id: authStore.username,
      description: 'IC token generated from member dashboard',
    })
  } catch (e) {
    generateError.value = e instanceof Error ? e.message : String(e)
  } finally {
    generating.value = false
  }
}

async function copyText(value: string, flag: 'gateway' | 'token') {
  try {
    await navigator.clipboard.writeText(value)
    if (flag === 'gateway') {
      copiedGateway.value = true
      setTimeout(() => {
        copiedGateway.value = false
      }, 1500)
    } else {
      copiedToken.value = true
      setTimeout(() => {
        copiedToken.value = false
      }, 1500)
    }
  } catch {
    generateError.value = 'Failed to copy to clipboard'
  }
}
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-6 py-10">
    <div class="space-y-1">
      <h1 class="text-2xl font-semibold">
        Welcome
        <span v-if="workspace">to {{ workspace.workspace_name }}</span>
      </h1>
      <p class="text-sm text-muted-foreground">Your access details for the Iron Cage gateway.</p>
    </div>

    <Alert
      v-if="workspaceError"
      variant="destructive"
    >
      <AlertDescription>{{ workspaceError }}</AlertDescription>
    </Alert>

    <Card>
      <CardHeader>
        <CardTitle>Gateway URL</CardTitle>
        <CardDescription> Point your SDK or HTTP client to this base URL. </CardDescription>
      </CardHeader>
      <CardContent class="space-y-3">
        <div class="rounded-md border bg-muted/30 p-3 font-mono text-sm break-all">
          {{ gatewayUrl }}
        </div>
        <Button
          variant="outline"
          size="sm"
          @click="copyText(gatewayUrl, 'gateway')"
        >
          <component
            :is="copiedGateway ? Check : Copy"
            class="size-4"
          />
          {{ copiedGateway ? 'Copied' : 'Copy URL' }}
        </Button>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>IC Token</CardTitle>
        <CardDescription>
          Use this token as the bearer credential for gateway requests. Shown once - copy it now.
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-3">
        <div
          v-if="!icToken"
          class="space-y-3"
        >
          <p class="text-sm text-muted-foreground">
            No token yet. Generate one to start using the gateway.
          </p>
          <Alert
            v-if="generateError"
            variant="destructive"
          >
            <AlertDescription>{{ generateError }}</AlertDescription>
          </Alert>
          <Button
            :disabled="generating"
            @click="generateIcToken"
          >
            {{ generating ? 'Generating...' : 'Generate Token' }}
          </Button>
        </div>

        <div
          v-else
          class="space-y-3"
        >
          <div class="rounded-md border bg-muted/30 p-3 font-mono text-sm break-all">
            {{ icToken.token }}
          </div>
          <Button
            variant="outline"
            size="sm"
            @click="copyText(icToken.token, 'token')"
          >
            <component
              :is="copiedToken ? Check : Copy"
              class="size-4"
            />
            {{ copiedToken ? 'Copied' : 'Copy Token' }}
          </Button>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Default Model</CardTitle>
        <CardDescription>
          The model that gateway requests resolve to when none is specified.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div
          v-if="workspaceLoading"
          class="text-sm text-muted-foreground"
        >
          Loading...
        </div>
        <Badge
          v-else-if="workspace?.default_model"
          variant="secondary"
        >
          {{ workspace.default_model }}
        </Badge>
        <p
          v-else
          class="text-sm text-muted-foreground"
        >
          Not configured. Ask your admin to set a default model.
        </p>
      </CardContent>
    </Card>
  </div>
</template>
