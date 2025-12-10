<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type TokenMetadata, type CreateTokenResponse } from '../composables/useApi'
import { useAuthStore } from '../stores/auth'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Alert, AlertDescription } from '@/components/ui/alert'

const api = useApi()
const authStore = useAuthStore()
const queryClient = useQueryClient()

const showCreateModal = ref(false)
const showTokenModal = ref(false)
const newTokenData = ref<CreateTokenResponse | null>(null)
const projectId = ref('')
const description = ref('')
const createError = ref('')

// Fetch tokens
const { data: tokens, isLoading, error, refetch } = useQuery({
  queryKey: ['tokens'],
  queryFn: () => api.getTokens(),
})

// Create token mutation
const createMutation = useMutation({
  mutationFn: (data: { user_id: string; project_id?: string; description?: string }) =>
    api.createToken(data),
  onSuccess: (data) => {
    newTokenData.value = data
    showCreateModal.value = false
    showTokenModal.value = true
    projectId.value = ''
    description.value = ''
    createError.value = ''
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  },
  onError: (err) => {
    createError.value = err instanceof Error ? err.message : 'Failed to create token'
  },
})

// Rotate token mutation
const rotateMutation = useMutation({
  mutationFn: (id: number) => api.rotateToken(id),
  onSuccess: (data) => {
    newTokenData.value = data
    showTokenModal.value = true
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  },
})

// Revoke token mutation
const revokeMutation = useMutation({
  mutationFn: (id: number) => api.revokeToken(id),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  },
})

function handleCreateToken() {
  createError.value = ''
  createMutation.mutate({
    user_id: authStore.username || 'default',
    project_id: projectId.value || undefined,
    description: description.value || undefined,
  })
}

function handleRotateToken(token: TokenMetadata) {
  if (confirm(`Rotate token ${token.id}? The old token will be revoked.`)) {
    rotateMutation.mutate(token.id)
  }
}

function handleRevokeToken(token: TokenMetadata) {
  if (confirm(`Revoke token ${token.id}? This action cannot be undone.`)) {
    revokeMutation.mutate(token.id)
  }
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString()
}

function copyToken(token: string) {
  navigator.clipboard.writeText(token)
}
</script>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-gray-900">Token Management</h1>
      <Button @click="showCreateModal = true">
        Generate New Token
      </Button>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading tokens...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading tokens: {{ error.message }}</p>
      <Button @click="() => refetch()" variant="secondary" class="mt-4">
        Retry
      </Button>
    </div>

    <!-- Tokens table -->
    <div v-else-if="tokens && tokens.length > 0" class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Provider
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Description
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Created
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          <tr v-for="token in tokens" :key="token.id">
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ token.id }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              <Badge variant="outline">{{ token.provider || '-' }}</Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ token.name || '-' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ formatDate(token.created_at) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <Badge :variant="token.is_active ? 'default' : 'destructive'">
                {{ token.is_active ? 'Active' : 'Revoked' }}
              </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
              <Button
                v-if="token.is_active"
                @click="handleRotateToken(token)"
                :disabled="rotateMutation.isPending.value"
                variant="ghost"
                size="sm"
              >
                Rotate
              </Button>
              <Button
                v-if="token.is_active"
                @click="handleRevokeToken(token)"
                :disabled="revokeMutation.isPending.value"
                variant="ghost"
                size="sm"
                class="text-destructive hover:text-destructive"
              >
                Revoke
              </Button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else class="bg-white rounded-lg shadow p-6 text-center">
      <p class="text-gray-600 mb-4">No tokens found</p>
      <Button @click="showCreateModal = true">
        Generate First Token
      </Button>
    </div>

    <!-- Create token modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Generate New Token</DialogTitle>
          <DialogDescription>
            Create a new API token with optional project ID and description.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="createError" variant="destructive" class="mb-4">
          <AlertDescription>{{ createError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="project">Project ID (optional)</Label>
            <Input
              id="project"
              v-model="projectId"
              placeholder="my-project"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-2">
            <Label for="description">Description (optional)</Label>
            <Input
              id="description"
              v-model="description"
              placeholder="Production API token"
              :disabled="createMutation.isPending.value"
            />
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="showCreateModal = false"
            :disabled="createMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="handleCreateToken"
            :disabled="createMutation.isPending.value"
          >
            {{ createMutation.isPending.value ? 'Generating...' : 'Generate Token' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- New token modal -->
    <Dialog v-model:open="showTokenModal">
      <DialogContent class="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Token Generated</DialogTitle>
          <DialogDescription>
            Save this token now. You won't be able to see it again!
          </DialogDescription>
        </DialogHeader>

        <Alert variant="default" class="bg-yellow-50 border-yellow-400">
          <AlertDescription class="text-yellow-800">
            <strong>Important:</strong> Save this token now. You won't be able to see it again!
          </AlertDescription>
        </Alert>

        <div v-if="newTokenData" class="space-y-4 py-4">
          <div class="space-y-2">
            <Label>Token</Label>
            <div class="flex space-x-2">
              <Input
                v-model="newTokenData.token"
                readonly
                class="font-mono text-sm bg-muted"
              />
              <Button
                @click="copyToken(newTokenData.token)"
                variant="secondary"
              >
                Copy
              </Button>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span class="text-muted-foreground">ID:</span>
              <span class="ml-2 font-medium">{{ newTokenData.id }}</span>
            </div>
            <div>
              <span class="text-muted-foreground">User:</span>
              <span class="ml-2 font-medium">{{ newTokenData.user_id }}</span>
            </div>
            <div v-if="newTokenData.provider">
              <span class="text-muted-foreground">Provider:</span>
              <span class="ml-2 font-medium">{{ newTokenData.provider }}</span>
            </div>
            <div v-if="newTokenData.description">
              <span class="text-muted-foreground">Description:</span>
              <span class="ml-2 font-medium">{{ newTokenData.description }}</span>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button @click="showTokenModal = false">
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
