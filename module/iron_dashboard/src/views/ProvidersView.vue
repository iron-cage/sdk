<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type ProviderKey, type ProviderType } from '../composables/useApi'
import { Button } from '@/components/ui/button'
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
import { Badge } from '@/components/ui/badge'

const api = useApi()
const queryClient = useQueryClient()

const showCreateModal = ref(false)
const showEditModal = ref(false)
const editingKey = ref<ProviderKey | null>(null)

// Form fields
const provider = ref<ProviderType>('openai')
const apiKey = ref('')
const baseUrl = ref('')
const description = ref('')
const isEnabled = ref(true)

// Error states
const createError = ref('')
const editError = ref('')

// Fetch provider keys
const { data: providerKeys, isLoading, error, refetch } = useQuery({
  queryKey: ['providerKeys'],
  queryFn: () => api.getProviderKeys(),
})

// Create provider key mutation
const createMutation = useMutation({
  mutationFn: (data: { provider: ProviderType; api_key: string; base_url?: string; description?: string }) =>
    api.createProviderKey(data),
  onSuccess: () => {
    showCreateModal.value = false
    resetForm()
    queryClient.invalidateQueries({ queryKey: ['providerKeys'] })
  },
  onError: (err) => {
    createError.value = err instanceof Error ? err.message : 'Failed to create provider key'
  },
})

// Update provider key mutation
const updateMutation = useMutation({
  mutationFn: (data: { id: number; base_url?: string; description?: string; is_enabled?: boolean }) =>
    api.updateProviderKey(data.id, { base_url: data.base_url, description: data.description, is_enabled: data.is_enabled }),
  onSuccess: () => {
    showEditModal.value = false
    editingKey.value = null
    queryClient.invalidateQueries({ queryKey: ['providerKeys'] })
  },
  onError: (err) => {
    editError.value = err instanceof Error ? err.message : 'Failed to update provider key'
  },
})

// Delete provider key mutation
const deleteMutation = useMutation({
  mutationFn: (id: number) => api.deleteProviderKey(id),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['providerKeys'] })
  },
})

// Toggle enabled state
const toggleMutation = useMutation({
  mutationFn: (data: { id: number; is_enabled: boolean }) =>
    api.updateProviderKey(data.id, { is_enabled: data.is_enabled }),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['providerKeys'] })
  },
})

function resetForm() {
  provider.value = 'openai'
  apiKey.value = ''
  baseUrl.value = ''
  description.value = ''
  isEnabled.value = true
  createError.value = ''
  editError.value = ''
}

function handleCreateKey() {
  createError.value = ''

  if (!apiKey.value.trim()) {
    createError.value = 'API key is required'
    return
  }

  createMutation.mutate({
    provider: provider.value,
    api_key: apiKey.value,
    base_url: baseUrl.value || undefined,
    description: description.value || undefined,
  })
}

function openEditModal(key: ProviderKey) {
  editingKey.value = key
  baseUrl.value = key.base_url || ''
  description.value = key.description || ''
  isEnabled.value = key.is_enabled
  editError.value = ''
  showEditModal.value = true
}

function handleUpdateKey() {
  if (!editingKey.value) return
  editError.value = ''

  updateMutation.mutate({
    id: editingKey.value.id,
    base_url: baseUrl.value || undefined,
    description: description.value || undefined,
    is_enabled: isEnabled.value,
  })
}

function handleDeleteKey(key: ProviderKey) {
  if (confirm(`Delete ${key.provider} key? This action cannot be undone.`)) {
    deleteMutation.mutate(key.id)
  }
}

function handleToggleEnabled(key: ProviderKey) {
  toggleMutation.mutate({ id: key.id, is_enabled: !key.is_enabled })
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString()
}

function getProviderLabel(providerType: ProviderType): string {
  return providerType === 'openai' ? 'OpenAI' : 'Anthropic'
}

function getProviderBadgeClass(providerType: ProviderType): string {
  return providerType === 'openai' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'
}
</script>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-gray-900">AI Provider Keys</h1>
      <Button @click="showCreateModal = true">
        Add Provider Key
      </Button>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading provider keys...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading provider keys: {{ (error as Error).message }}</p>
      <Button @click="() => refetch()" variant="secondary" class="mt-4">
        Retry
      </Button>
    </div>

    <!-- Provider keys table -->
    <div v-else-if="providerKeys && providerKeys.length > 0" class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Provider
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Description
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              API Key
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Created
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          <tr v-for="key in providerKeys" :key="key.id">
            <td class="px-6 py-4 whitespace-nowrap">
              <Badge :class="getProviderBadgeClass(key.provider)">
                {{ getProviderLabel(key.provider) }}
              </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ key.description || '-' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-500">
              {{ key.masked_key }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <Button
                @click="handleToggleEnabled(key)"
                :disabled="toggleMutation.isPending.value"
                :variant="key.is_enabled ? 'default' : 'outline'"
                size="sm"
                :class="key.is_enabled ? 'bg-green-600 hover:bg-green-700' : ''"
              >
                {{ key.is_enabled ? 'Enabled' : 'Disabled' }}
              </Button>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ formatDate(key.created_at) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
              <Button
                @click="openEditModal(key)"
                :disabled="updateMutation.isPending.value"
                variant="ghost"
                size="sm"
              >
                Edit
              </Button>
              <Button
                @click="handleDeleteKey(key)"
                :disabled="deleteMutation.isPending.value"
                variant="ghost"
                size="sm"
                class="text-destructive hover:text-destructive"
              >
                Delete
              </Button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else class="bg-white rounded-lg shadow p-6 text-center">
      <p class="text-gray-600 mb-4">No AI provider keys configured</p>
      <p class="text-sm text-gray-500 mb-4">Add your OpenAI or Anthropic API keys to start using AI services.</p>
      <Button @click="showCreateModal = true">
        Add First Provider Key
      </Button>
    </div>

    <!-- Create provider key modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Add Provider Key</DialogTitle>
          <DialogDescription>
            Add an API key for OpenAI or Anthropic. The key will be encrypted and stored securely.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="createError" variant="destructive">
          <AlertDescription>{{ createError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="provider">Provider</Label>
            <Select v-model="provider" :disabled="createMutation.isPending.value">
              <SelectTrigger>
                <SelectValue placeholder="Select provider" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="openai">OpenAI</SelectItem>
                <SelectItem value="anthropic">Anthropic</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-2">
            <Label for="apiKey">API Key</Label>
            <Input
              id="apiKey"
              v-model="apiKey"
              type="password"
              :placeholder="provider === 'openai' ? 'sk-proj-...' : 'sk-ant-...'"
              :disabled="createMutation.isPending.value"
            />
            <p class="text-xs text-gray-500">
              Your API key will be encrypted and never shown again after creation.
            </p>
          </div>

          <div class="space-y-2">
            <Label for="baseUrl">Base URL (optional)</Label>
            <Input
              id="baseUrl"
              v-model="baseUrl"
              placeholder="https://api.openai.com/v1"
              :disabled="createMutation.isPending.value"
            />
            <p class="text-xs text-gray-500">
              Custom endpoint for proxy or self-hosted deployments.
            </p>
          </div>

          <div class="space-y-2">
            <Label for="description">Description (optional)</Label>
            <Input
              id="description"
              v-model="description"
              placeholder="e.g., Production key, Development key"
              :disabled="createMutation.isPending.value"
            />
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="showCreateModal = false; resetForm()"
            :disabled="createMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="handleCreateKey"
            :disabled="createMutation.isPending.value"
          >
            {{ createMutation.isPending.value ? 'Adding...' : 'Add Key' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Edit provider key modal -->
    <Dialog v-model:open="showEditModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Edit Provider Key</DialogTitle>
          <DialogDescription>
            Update the description or base URL. The API key cannot be changed - delete and create a new key instead.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="editError" variant="destructive">
          <AlertDescription>{{ editError }}</AlertDescription>
        </Alert>

        <div v-if="editingKey" class="space-y-4 py-4">
          <div class="space-y-2">
            <Label>Provider</Label>
            <p class="text-sm text-gray-900">{{ getProviderLabel(editingKey.provider) }}</p>
          </div>

          <div class="space-y-2">
            <Label for="editBaseUrl">Base URL (optional)</Label>
            <Input
              id="editBaseUrl"
              v-model="baseUrl"
              placeholder="https://api.openai.com/v1"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="space-y-2">
            <Label for="editDescription">Description (optional)</Label>
            <Input
              id="editDescription"
              v-model="description"
              placeholder="e.g., Production key, Development key"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="flex items-center space-x-2">
            <input
              id="editEnabled"
              type="checkbox"
              v-model="isEnabled"
              :disabled="updateMutation.isPending.value"
              class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
            />
            <Label for="editEnabled">Enabled</Label>
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="showEditModal = false; editingKey = null"
            :disabled="updateMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="handleUpdateKey"
            :disabled="updateMutation.isPending.value"
          >
            {{ updateMutation.isPending.value ? 'Updating...' : 'Update Key' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
