<script setup lang="ts">
import { ref } from 'vue'
import { useRoute } from 'vue-router'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Alert, AlertDescription } from '@/components/ui/alert'

const route = useRoute()
const api = useApi()
const authStore = useAuthStore()
const queryClient = useQueryClient()

const agentId = parseInt(route.params.agentId as string)

const showCreateModal = ref(false)
const showTokenModal = ref(false)
const showSwitchProviderModal = ref(false)
const newTokenData = ref<CreateTokenResponse | null>(null)
const selectedProvider = ref('')
const description = ref('')
const createError = ref('')
const targetUserId = ref('')
const switchingToken = ref<TokenMetadata | null>(null)
const newProvider = ref('')

// Fetch agent details
const { data: agent } = useQuery({
  queryKey: ['agent', agentId],
  queryFn: () => api.getAgent(agentId),
})

// Fetch tokens for this agent
const { data: tokens, isLoading, error, refetch } = useQuery({
  queryKey: ['agent-tokens', agentId],
  queryFn: () => api.getAgentTokens(agentId),
})

// Fetch users list (for admin dropdown)
const { data: users } = useQuery({
  queryKey: ['users'],
  queryFn: () => api.getUsers(),
  enabled: authStore.isAdmin, // Only fetch if user is admin
})

// Create token mutation
const createMutation = useMutation({
  mutationFn: (data: { agent_id: number; user_id: string; provider: string; description?: string }) =>
    api.createAgentToken(data),
  onSuccess: (data) => {
    newTokenData.value = data
    showCreateModal.value = false
    showTokenModal.value = true
    selectedProvider.value = ''
    description.value = ''
    targetUserId.value = ''
    createError.value = ''
    queryClient.invalidateQueries({ queryKey: ['agent-tokens', agentId] })
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
    queryClient.invalidateQueries({ queryKey: ['agent-tokens', agentId] })
  },
})

// Revoke token mutation
const revokeMutation = useMutation({
  mutationFn: (id: number) => api.revokeToken(id),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['agent-tokens', agentId] })
  },
})

// Switch provider mutation
const switchProviderMutation = useMutation({
  mutationFn: (data: { tokenId: number; provider: string }) =>
    api.updateTokenProvider(data.tokenId, data.provider),
  onSuccess: () => {
    showSwitchProviderModal.value = false
    switchingToken.value = null
    newProvider.value = ''
    queryClient.invalidateQueries({ queryKey: ['agent-tokens', agentId] })
  },
})

function handleCreateToken() {
  if (!selectedProvider.value) {
    createError.value = 'Provider is required'
    return
  }

  // Admin can create tokens for other users
  const userId = authStore.isAdmin && targetUserId.value 
    ? targetUserId.value 
    : authStore.username || 'default'

  createError.value = ''
  createMutation.mutate({
    agent_id: agentId,
    user_id: userId,
    provider: selectedProvider.value,
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

function openSwitchProviderModal(token: TokenMetadata) {
  switchingToken.value = token
  newProvider.value = token.provider || ''
  showSwitchProviderModal.value = true
}

function handleSwitchProvider() {
  if (!switchingToken.value || !newProvider.value) return
  
  switchProviderMutation.mutate({
    tokenId: switchingToken.value.id,
    provider: newProvider.value,
  })
}

function canManageToken(token: TokenMetadata): boolean {
  // Admin can manage all tokens
  if (authStore.isAdmin) return true
  // Users can only manage their own tokens
  return token.user_id === authStore.username
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
      <div>
        <h1 class="text-2xl font-bold text-gray-900">
          Agent Tokens: {{ agent?.name || 'Loading...' }}
        </h1>
        <router-link to="/agents" class="text-sm text-blue-600 hover:underline">
          &larr; Back to Agents
        </router-link>
      </div>
      <Button v-if="authStore.isAdmin" @click="showCreateModal = true">
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
            <th v-if="authStore.isAdmin" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              User
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
            <td v-if="authStore.isAdmin" class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ token.user_id }}
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
                v-if="token.is_active && agent && agent.providers.length > 1 && canManageToken(token)"
                @click="openSwitchProviderModal(token)"
                :disabled="switchProviderMutation.isPending.value"
                variant="ghost"
                size="sm"
              >
                Switch
              </Button>
              <Button
                v-if="token.is_active && canManageToken(token)"
                @click="handleRotateToken(token)"
                :disabled="rotateMutation.isPending.value"
                variant="ghost"
                size="sm"
              >
                Rotate
              </Button>
              <Button
                v-if="token.is_active && canManageToken(token)"
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
      <p class="text-gray-600 mb-4">No tokens found for this agent</p>
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
            Create a new API token for this agent.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="createError" variant="destructive" class="mb-4">
          <AlertDescription>{{ createError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div v-if="authStore.isAdmin" class="space-y-2">
            <Label for="targetUser">User (Admin only)</Label>
            <Select v-model="targetUserId" :disabled="createMutation.isPending.value">
              <SelectTrigger>
                <SelectValue placeholder="Select user (or leave empty for yourself)" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem :value="authStore.username">
                  {{ authStore.username }} (yourself)
                </SelectItem>
                <SelectItem 
                  v-for="user in users || []" 
                  :key="user.username" 
                  :value="user.username"
                >
                  {{ user.username }}
                </SelectItem>
              </SelectContent>
            </Select>
            <p class="text-xs text-gray-500">
              Select a user to create the token for.
            </p>
          </div>

          <div class="space-y-2">
            <Label for="provider">Provider</Label>
            <Select v-model="selectedProvider" :disabled="createMutation.isPending.value">
              <SelectTrigger>
                <SelectValue placeholder="Select a provider" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem 
                  v-for="provider in agent?.providers || []" 
                  :key="provider" 
                  :value="provider"
                >
                  {{ provider }}
                </SelectItem>
              </SelectContent>
            </Select>
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

    <!-- Switch Provider Modal -->
    <Dialog v-model:open="showSwitchProviderModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Switch Provider</DialogTitle>
          <DialogDescription>
            Change the AI provider for this token.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="newProvider">New Provider</Label>
            <Select v-model="newProvider" :disabled="switchProviderMutation.isPending.value">
              <SelectTrigger>
                <SelectValue placeholder="Select a provider" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem 
                  v-for="provider in agent?.providers || []" 
                  :key="provider" 
                  :value="provider"
                >
                  {{ provider }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="showSwitchProviderModal = false"
            :disabled="switchProviderMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="handleSwitchProvider"
            :disabled="switchProviderMutation.isPending.value"
          >
            {{ switchProviderMutation.isPending.value ? 'Switching...' : 'Switch Provider' }}
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
