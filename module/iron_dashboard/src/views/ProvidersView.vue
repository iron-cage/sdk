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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import PageLayout from '@/components/PageLayout.vue'
import DataTable from '@/components/DataTable.vue'

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

// Note: Tokens are now user-centric and provider-based, not project-based

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
  return providerType === 'openai' ? 'bg-muted text-foreground' : 'bg-muted text-foreground'
}
</script>

<template>
  <PageLayout title="AI Provider Keys">
    <template #actions>
      <Button @click="showCreateModal = true">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
        Add Provider Key
      </Button>
    </template>

    <DataTable
      :columns="[
        { label: 'Provider' },
        { label: 'Description' },
        { label: 'API Key' },
        { label: 'Status' },
        { label: 'Created' },
        { label: 'Actions', align: 'right' },
      ]"
      :is-loading="isLoading"
      :error="error as Error | null"
      :is-empty="!providerKeys || providerKeys.length === 0"
      loading-text="Loading provider keys..."
      :on-retry="() => refetch()"
    >
      <template #empty>
        <p class="text-muted-foreground mb-2">No AI provider keys configured</p>
        <p class="text-base text-muted-foreground mb-4">Add your OpenAI or Anthropic API keys to start using AI services.</p>
        <Button @click="showCreateModal = true"><svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>Add First Provider Key</Button>
      </template>

      <tr v-for="key in providerKeys" :key="key.id">
        <td class="px-3 sm:px-6 py-4 whitespace-nowrap">
          <Badge :class="getProviderBadgeClass(key.provider)">{{ getProviderLabel(key.provider) }}</Badge>
        </td>
        <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">{{ key.description || '-' }}</td>
        <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base font-mono text-muted-foreground">{{ key.masked_key }}</td>
        <td class="px-3 sm:px-6 py-4 whitespace-nowrap">
          <Button
            @click="handleToggleEnabled(key)"
            :disabled="toggleMutation.isPending.value"
            :variant="key.is_enabled ? 'default' : 'outline'"
            size="sm"
            :class="key.is_enabled ? '' : 'text-muted-foreground'"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="key.is_enabled ? 'M5 13l4 4L19 7' : 'M6 18L18 6M6 6l12 12'" /></svg>
            {{ key.is_enabled ? 'Enabled' : 'Disabled' }}
          </Button>
        </td>
        <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-muted-foreground">{{ formatDate(key.created_at) }}</td>
        <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-right text-base font-medium">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm">
                <span class="sr-only">Open menu</span>
                <svg width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4"><path d="M3.625 7.5C3.625 8.12132 3.12132 8.625 2.5 8.625C1.87868 8.625 1.375 8.12132 1.375 7.5C1.375 6.87868 1.87868 6.375 2.5 6.375C3.12132 6.375 3.625 6.87868 3.625 7.5ZM8.625 7.5C8.625 8.12132 8.12132 8.625 7.5 8.625C6.87868 8.625 6.375 8.12132 6.375 7.5C6.375 6.87868 6.87868 6.375 7.5 6.375C8.12132 6.375 8.625 6.87868 8.625 7.5ZM13.625 7.5C13.625 8.12132 13.1213 8.625 12.5 8.625C11.8787 8.625 11.375 8.12132 11.375 7.5C11.375 6.87868 11.8787 6.375 12.5 6.375C13.1213 6.375 13.625 6.87868 13.625 7.5Z" fill="currentColor" fill-rule="evenodd" clip-rule="evenodd"></path></svg>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem @click="openEditModal(key)" :disabled="updateMutation.isPending.value">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
                Edit
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem @click="handleDeleteKey(key)" :disabled="deleteMutation.isPending.value" class="text-destructive">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </td>
      </tr>
    </DataTable>

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
            <p class="text-xs text-muted-foreground">
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
            <p class="text-xs text-muted-foreground">
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
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="handleCreateKey"
            :disabled="createMutation.isPending.value"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
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
            <p class="text-base text-foreground">{{ getProviderLabel(editingKey.provider) }}</p>
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
              class="h-4 w-4 rounded border-border text-primary focus:ring-ring"
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
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="handleUpdateKey"
            :disabled="updateMutation.isPending.value"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            {{ updateMutation.isPending.value ? 'Updating...' : 'Update Key' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </PageLayout>
</template>
