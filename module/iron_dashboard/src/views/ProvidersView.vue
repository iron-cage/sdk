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
import { toast } from 'vue-sonner'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { formatDate, formatCostUsd } from '@/lib/formatters'
import { getProviderLabel, getProviderKeyPlaceholder } from '@/lib/providers'
import { useConfirm } from '@/composables/useConfirm'
import ProviderBadge from '@/components/ProviderBadge.vue'
import IconPlus from '@/components/icons/IconPlus.vue'
import IconX from '@/components/icons/IconX.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconDotsHorizontal from '@/components/icons/IconDotsHorizontal.vue'
import IconTrash from '@/components/icons/IconTrash.vue'
import IconEdit from '@/components/icons/IconEdit.vue'
import PageLayout from '@/components/PageLayout.vue'
import DataTable from '@/components/DataTable.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import Switch from '@/components/ui/switch/Switch.vue'

const api = useApi()
const queryClient = useQueryClient()

const showCreateModal = ref(false)
const showEditModal = ref(false)
const { showConfirmModal, confirmTitle, confirmDescription, confirmLabel, confirmCallback, openConfirm } = useConfirm()
const editingKey = ref<ProviderKey | null>(null)

// Form fields
const provider = ref<ProviderType>('openai')
const apiKey = ref('')
const alias = ref('')
const baseUrl = ref('')
const description = ref('')
const isEnabled = ref(true)


// Fetch provider keys
const { data: providerKeys, isLoading, error, refetch } = useQuery({
  queryKey: ['providerKeys'],
  queryFn: () => api.getProviderKeys(),
})

// Create provider key mutation
const createMutation = useMutation({
  mutationFn: (data: { provider: ProviderType; api_key: string; alias?: string; base_url?: string; description?: string }) =>
    api.createProviderKey(data),
  onSuccess: () => {
    showCreateModal.value = false
    resetForm()
    queryClient.invalidateQueries({ queryKey: ['providerKeys'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create provider key')
  },
})

// Update provider key mutation
const updateMutation = useMutation({
  mutationFn: (data: { id: number; alias?: string; base_url?: string; description?: string; is_enabled?: boolean }) =>
    api.updateProviderKey(data.id, { alias: data.alias, base_url: data.base_url, description: data.description, is_enabled: data.is_enabled }),
  onSuccess: () => {
    showEditModal.value = false
    editingKey.value = null
    queryClient.invalidateQueries({ queryKey: ['providerKeys'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to update provider key')
  },
})

// Delete provider key mutation
const deleteMutation = useMutation({
  mutationFn: (id: number) => api.deleteProviderKey(id),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['providerKeys'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to delete provider key')
  },
})

// Toggle enabled state — optimistic update so the switch flips immediately
const toggleMutation = useMutation({
  mutationFn: (data: { id: number; is_enabled: boolean }) =>
    api.updateProviderKey(data.id, { is_enabled: data.is_enabled }),
  onMutate: async (data) => {
    await queryClient.cancelQueries({ queryKey: ['providerKeys'] })
    const previous = queryClient.getQueryData<ProviderKey[]>(['providerKeys'])
    queryClient.setQueryData<ProviderKey[]>(['providerKeys'], old =>
      old?.map(k => k.id === data.id ? { ...k, is_enabled: data.is_enabled } : k)
    )
    return { previous }
  },
  onError: (_err, _vars, context) => {
    if (context?.previous) queryClient.setQueryData(['providerKeys'], context.previous)
    toast.error('Failed to update provider key')
  },
  onSettled: () => {
    queryClient.invalidateQueries({ queryKey: ['providerKeys'] })
  },
})



function resetForm() {
  provider.value = 'openai'
  apiKey.value = ''
  alias.value = ''
  baseUrl.value = ''
  description.value = ''
  isEnabled.value = true
}

function handleCreateKey() {
  if (!apiKey.value.trim()) {
    toast.error('API key is required')
    return
  }

  createMutation.mutate({
    provider: provider.value,
    api_key: apiKey.value,
    alias: alias.value || undefined,
    base_url: baseUrl.value || undefined,
    description: description.value || undefined,
  })
}

function openEditModal(key: ProviderKey) {
  editingKey.value = key
  alias.value = key.alias || ''
  baseUrl.value = key.base_url || ''
  description.value = key.description || ''
  isEnabled.value = key.is_enabled
  showEditModal.value = true
}

function handleUpdateKey() {
  if (!editingKey.value) return
  updateMutation.mutate({
    id: editingKey.value.id,
    alias: alias.value || undefined,
    base_url: baseUrl.value || undefined,
    description: description.value || undefined,
    is_enabled: isEnabled.value,
  })
}

function handleDeleteKey(key: ProviderKey) {
  openConfirm(
    'Delete Provider Key',
    `Delete the ${getProviderLabel(key.provider)} key? This action cannot be undone.`,
    'Delete',
    () => deleteMutation.mutate(key.id),
  )
}

function handleToggleEnabled(key: ProviderKey) {
  toggleMutation.mutate({ id: key.id, is_enabled: !key.is_enabled })
}

const typedError = error as unknown as Error | null

</script>

<template>
  <PageLayout title="AI Provider Keys">
    <template #actions>
      <Button @click="showCreateModal = true">
        <IconPlus />
        Add Provider Key
      </Button>
    </template>

    <DataTable
      :columns="[
        { label: 'Provider' },
        { label: 'Name' },
        { label: 'Description' },
        { label: 'API Key' },
        { label: 'Spend (all-time)' },
        { label: 'Status' },
        { label: 'Created' },
        { label: 'Actions', align: 'right' },
      ]"
      :is-loading="isLoading"
      :error="typedError"
      :is-empty="!providerKeys || providerKeys.length === 0"
      loading-text="Loading provider keys..."
      :on-retry="() => refetch()"
    >
      <template #empty>
        <p class="text-muted-foreground mb-2">No AI provider keys configured</p>
        <p class="text-base text-muted-foreground mb-4">Add your OpenAI or Anthropic API keys to start using AI services.</p>
        <Button @click="showCreateModal = true"><IconPlus />Add First Provider Key</Button>
      </template>

      <tr v-for="key in providerKeys" :key="key.id">
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap">
          <ProviderBadge :provider="key.provider" />
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">{{ key.alias || '-' }}</td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">{{ key.description || '-' }}</td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base font-mono text-muted-foreground">{{ key.masked_key }}</td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">
          {{ formatCostUsd(key.total_spend_usd, 2) }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap relative -left-3">
          <Button
            variant="outline"
            size="sm"
            :disabled="toggleMutation.isPending.value"
            @click="handleToggleEnabled(key)"
          >
            <IconCheck v-if="key.is_enabled" class="text-success" />
            <IconX v-else class="text-muted-foreground" />
            {{ key.is_enabled ? 'Enabled' : 'Disabled' }}
          </Button>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">{{ formatDate(key.created_at) }}</td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-right text-base font-medium">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm">
                <span class="sr-only">Open menu</span>
                <IconDotsHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem :disabled="updateMutation.isPending.value" @click="openEditModal(key)">
                <IconEdit />
                Edit
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem :disabled="deleteMutation.isPending.value" class="text-destructive" @click="handleDeleteKey(key)">
                <IconTrash />
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

        <div class="space-y-4">
          <div class="space-y-1.5">
            <Label for="provider">Provider</Label>
            <Select v-model="provider" :disabled="createMutation.isPending.value">
              <SelectTrigger>
                <SelectValue placeholder="Select provider" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="openai">{{ getProviderLabel('openai') }}</SelectItem>
                <SelectItem value="anthropic">{{ getProviderLabel('anthropic') }}</SelectItem>
                <SelectItem value="gemini">{{ getProviderLabel('gemini') }}</SelectItem>
                <SelectItem value="xai">{{ getProviderLabel('xai') }}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-1.5">
            <Label for="apiKey">API Key</Label>
            <Input
              id="apiKey"
              v-model="apiKey"
              type="password"
              :placeholder="getProviderKeyPlaceholder(provider)"
              :disabled="createMutation.isPending.value"
            />
            <p class="text-xs text-muted-foreground">
              Your API key will be encrypted and never shown again after creation.
            </p>
          </div>

          <div class="space-y-1.5">
            <Label for="alias">Name / Alias (optional)</Label>
            <Input
              id="alias"
              v-model="alias"
              placeholder="e.g., Production, Team A key"
              :disabled="createMutation.isPending.value"
            />
            <p class="text-xs text-muted-foreground">
              A friendly name shown in the agent selector instead of the provider type.
            </p>
          </div>

          <div class="space-y-1.5">
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

          <div class="space-y-1.5">
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
            :disabled="createMutation.isPending.value"
            variant="outline"
            @click="showCreateModal = false; resetForm()"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="createMutation.isPending.value"
            @click="handleCreateKey"
          >
            <IconPlus />
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

        <div v-if="editingKey" class="space-y-4">
          <div class="space-y-1.5">
            <Label>Provider</Label>
            <p class="text-base text-foreground">{{ getProviderLabel(editingKey.provider) }}</p>
          </div>

          <div class="space-y-1.5">
            <Label for="editAlias">Name / Alias (optional)</Label>
            <Input
              id="editAlias"
              v-model="alias"
              placeholder="e.g., Production, Team A key"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="editBaseUrl">Base URL (optional)</Label>
            <Input
              id="editBaseUrl"
              v-model="baseUrl"
              placeholder="https://api.openai.com/v1"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="editDescription">Description (optional)</Label>
            <Input
              id="editDescription"
              v-model="description"
              placeholder="e.g., Production key, Development key"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="flex items-center space-x-2">
            <Switch
              id="editEnabled"
              v-model="isEnabled"
              :disabled="updateMutation.isPending.value"
            />
            <Label for="editEnabled">Enabled</Label>
          </div>
        </div>

        <DialogFooter>
          <Button
            :disabled="updateMutation.isPending.value"
            variant="outline"
            @click="showEditModal = false; editingKey = null"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="updateMutation.isPending.value"
            @click="handleUpdateKey"
          >
            <IconCheck />
            {{ updateMutation.isPending.value ? 'Updating...' : 'Update Key' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <ConfirmDialog
      v-model:open="showConfirmModal"
      :title="confirmTitle"
      :description="confirmDescription"
      :confirm-label="confirmLabel"
      @confirm="confirmCallback?.()"
    />
  </PageLayout>
</template>
