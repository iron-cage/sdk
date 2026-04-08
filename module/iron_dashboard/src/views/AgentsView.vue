<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useQuery, useQueries, useMutation, useQueryClient } from '@tanstack/vue-query'
import { toast } from 'vue-sonner'

import { useApi } from '@/composables/useApi'
import { useAuthStore } from '@/stores/auth'
import { useConfirm } from '@/composables/useConfirm'
import { formatTimestamp } from '@/lib/formatters'

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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import StatusBadge from '@/components/StatusBadge.vue'
import IconPlus from '@/components/icons/IconPlus.vue'
import IconX from '@/components/icons/IconX.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconDotsHorizontal from '@/components/icons/IconDotsHorizontal.vue'
import IconTrash from '@/components/icons/IconTrash.vue'
import IconEdit from '@/components/icons/IconEdit.vue'
import IconRefresh from '@/components/icons/IconRefresh.vue'
import IconKey from '@/components/icons/IconKey.vue'
import IconBan from '@/components/icons/IconBan.vue'
import IconCopy from '@/components/icons/IconCopy.vue'
import PageLayout from '@/components/PageLayout.vue'
import DataTable from '@/components/DataTable.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'

import type { Agent, IcTokenStatus } from '@/composables/useApi'

const api = useApi()
const queryClient = useQueryClient()
const authStore = useAuthStore()

const showCreateModal = ref(false)
const showUpdateModal = ref(false)
const {
  showConfirmModal,
  confirmTitle,
  confirmDescription,
  confirmLabel,
  confirmVariant,
  confirmCallback,
  openConfirm,
} = useConfirm()
const name = ref('')
const selectedProviderKeyIds = ref<number[]>([])
const addingProviderKeyId = ref<string>('')
const initialBudgetUsd = ref<number | undefined>(undefined)
const selectedOwnerId = ref<string>('')
const selectedAgent = ref<Agent | null>(null)
const tokenActionLoadingId = ref<number | null>(null)
const showTokenDialog = ref(false)
const tokenDialogValue = ref('')
const tokenDialogAgentName = ref('')
const tokenDialogWarning = ref('')
const copyMessage = ref('')

// Fetch agents
const {
  data: agents,
  isLoading,
  error,
  refetch,
} = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

// Fetch IC token status per agent — one query per agent, managed by TanStack Query
const icTokenQueries = useQueries({
  queries: computed(() =>
    (agents.value ?? []).map((agent) => ({
      queryKey: ['ic-token-status', agent.id] as const,
      queryFn: () => api.getIcTokenStatus(agent.id),
      staleTime: 60_000,
    }))
  ),
})

const icTokenStatusLoading = computed(() => icTokenQueries.value.some((q) => q.isLoading))

function getIcTokenStatusFromQuery(agentId: number): IcTokenStatus | undefined {
  const idx = agents.value?.findIndex((a) => a.id === agentId) ?? -1
  return idx >= 0 ? icTokenQueries.value[idx]?.data : undefined
}

// Fetch providers for selection
const { data: providers } = useQuery({
  queryKey: ['providers'],
  queryFn: () => api.getProviderKeys(),
})

const availableProviderKeys = computed(() =>
  (providers.value ?? []).filter((p) => !selectedProviderKeyIds.value.includes(p.id))
)

function handleAddProviderKey() {
  const id = Number(addingProviderKeyId.value)
  if (id && !selectedProviderKeyIds.value.includes(id)) {
    selectedProviderKeyIds.value = [...selectedProviderKeyIds.value, id]
    addingProviderKeyId.value = ''
  }
}

function handleRemoveProviderKey(keyId: number) {
  selectedProviderKeyIds.value = selectedProviderKeyIds.value.filter((id) => id !== keyId)
}

const providerKeyMap = computed(() => new Map(providers.value?.map((p) => [p.id, p]) ?? []))

function providerKeyLabel(keyId: number): string {
  const key = providerKeyMap.value.get(keyId)
  if (!key) return `#${keyId}`
  return key.alias || key.provider
}

// Fetch users for owner selection (admin only)
const { data: users } = useQuery({
  queryKey: ['users-for-agents'],
  queryFn: () => api.getUsers({ is_active: true }),
  enabled: authStore.isAdmin,
})

// O(1) lookup map — avoids O(n×m) Array.find on every render cycle
const ownerMap = computed(() => new Map(users.value?.users.map((u) => [u.id, u]) ?? []))

function ownerEmail(ownerId: string | null | undefined): string {
  if (!ownerId) return '—'
  if (!users.value) return '—'
  const user = ownerMap.value.get(ownerId)
  return user?.email || user?.username || '—'
}

const showOwnerColumn = computed(() => authStore.isAdmin)

const tableColumns = computed(() => [
  { label: 'Name' },
  ...(showOwnerColumn.value ? [{ label: 'Owner' }] : []),
  { label: 'Providers' },
  { label: 'IC Token' },
  { label: 'Created' },
  { label: 'Actions', align: 'right' as const },
])

// Create agent mutation
const createMutation = useMutation({
  mutationFn: (data: {
    name: string
    providers: string[]
    provider_key_ids: number[]
    initial_budget_microdollars: number
    owner_id?: string
  }) => api.createAgent(data),
  onSuccess: () => {
    showCreateModal.value = false
    name.value = ''
    selectedProviderKeyIds.value = []
    addingProviderKeyId.value = ''
    initialBudgetUsd.value = undefined
    selectedOwnerId.value = ''
    queryClient.invalidateQueries({ queryKey: ['agents'] })
    toast.success('Agent created successfully')
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create agent')
  },
})

function resetUpdateForm() {
  selectedAgent.value = null
  name.value = ''
  selectedProviderKeyIds.value = []
  addingProviderKeyId.value = ''
  selectedOwnerId.value = ''
}

// Update agent mutation
const updateMutation = useMutation({
  mutationFn: (data: {
    id: number
    name: string
    providers: string[]
    provider_key_ids: number[]
    owner_id?: string
  }) => api.updateAgent(data),
  onSuccess: () => {
    showUpdateModal.value = false
    resetUpdateForm()
    queryClient.invalidateQueries({ queryKey: ['agents'] })
    toast.success('Agent updated successfully')
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to update agent')
  },
})

// Delete agent mutation
const deleteMutation = useMutation({
  mutationFn: (id: number) => api.deleteAgent(id),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['agents'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to delete agent')
  },
})

function handleCreateAgent() {
  if (!name.value) {
    toast.error('Name is required')
    return
  }

  if (selectedProviderKeyIds.value.length === 0) {
    toast.error('At least one provider key is required')
    return
  }

  if (!initialBudgetUsd.value || initialBudgetUsd.value <= 0) {
    toast.error('Initial budget (USD) is required and must be positive')
    return
  }

  const selectedKeys = (providers.value ?? []).filter((p) =>
    selectedProviderKeyIds.value.includes(p.id)
  )
  const uniqueProviders = [...new Set(selectedKeys.map((p) => p.provider))]
  const budgetMicros = Math.round(initialBudgetUsd.value * 1_000_000)

  createMutation.mutate({
    name: name.value,
    providers: uniqueProviders,
    provider_key_ids: selectedProviderKeyIds.value,
    initial_budget_microdollars: budgetMicros,
    owner_id: selectedOwnerId.value || undefined,
  })
}

function handleOpenUpdateModal(agent: Agent) {
  selectedAgent.value = agent
  name.value = agent.name
  selectedProviderKeyIds.value = [...(agent.provider_key_ids ?? [])]
  addingProviderKeyId.value = ''
  selectedOwnerId.value = agent.owner_id ?? ''
  showUpdateModal.value = true
}

function handleUpdateAgent() {
  if (!selectedAgent.value || !name.value) {
    toast.error('Name is required')
    return
  }

  if (selectedProviderKeyIds.value.length === 0) {
    toast.error('At least one provider key is required')
    return
  }

  const selectedKeys = (providers.value ?? []).filter((p) =>
    selectedProviderKeyIds.value.includes(p.id)
  )
  const uniqueProviders = [...new Set(selectedKeys.map((p) => p.provider))]

  updateMutation.mutate({
    id: selectedAgent.value.id,
    name: name.value,
    providers: uniqueProviders,
    provider_key_ids: selectedProviderKeyIds.value,
    owner_id: selectedOwnerId.value || undefined,
  })
}

function handleDeleteAgent(agent: Agent) {
  openConfirm(
    'Delete Agent',
    `Delete "${agent.name}"? This action cannot be undone.`,
    'Delete',
    () => deleteMutation.mutate(agent.id),
    'destructive'
  )
}

function getIcTokenStatus(agentId: number): IcTokenStatus | undefined {
  return getIcTokenStatusFromQuery(agentId)
}

const generateIcTokenMutation = useMutation({
  mutationFn: (agentId: number) => api.generateIcToken(agentId),
  onMutate: (agentId) => {
    tokenActionLoadingId.value = agentId
  },
  onSuccess: (response, agentId) => {
    queryClient.setQueryData(['ic-token-status', agentId], {
      agent_id: agentId,
      has_ic_token: true,
      created_at: response.created_at,
    })
    const agent = agents.value?.find((a) => a.id === agentId)
    tokenDialogAgentName.value = agent?.name ?? ''
    tokenDialogValue.value = response.ic_token
    tokenDialogWarning.value = response.warning
    copyMessage.value = ''
    showTokenDialog.value = true
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to generate IC token')
  },
  onSettled: () => {
    tokenActionLoadingId.value = null
  },
})

const regenerateIcTokenMutation = useMutation({
  mutationFn: (agentId: number) => api.regenerateIcToken(agentId),
  onMutate: (agentId) => {
    tokenActionLoadingId.value = agentId
  },
  onSuccess: (response, agentId) => {
    queryClient.setQueryData(['ic-token-status', agentId], {
      agent_id: agentId,
      has_ic_token: true,
      created_at: response.created_at,
    })
    const agent = agents.value?.find((a) => a.id === agentId)
    tokenDialogAgentName.value = agent?.name ?? ''
    tokenDialogValue.value = response.ic_token
    tokenDialogWarning.value = response.warning || 'Old IC token is now invalid.'
    copyMessage.value = ''
    showTokenDialog.value = true
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to regenerate IC token')
  },
  onSettled: () => {
    tokenActionLoadingId.value = null
  },
})

const revokeIcTokenMutation = useMutation({
  mutationFn: (agentId: number) => api.revokeIcToken(agentId),
  onMutate: (agentId) => {
    tokenActionLoadingId.value = agentId
  },
  onSuccess: (_data, agentId) => {
    queryClient.setQueryData(['ic-token-status', agentId], {
      agent_id: agentId,
      has_ic_token: false,
      created_at: null,
    })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to revoke IC token')
  },
  onSettled: () => {
    tokenActionLoadingId.value = null
  },
})

function handleGenerateIcToken(agent: Agent) {
  generateIcTokenMutation.mutate(agent.id)
}

function handleRegenerateIcToken(agent: Agent) {
  openConfirm(
    'Regenerate IC Token',
    `Regenerate IC token for ${agent.name}? The current token will be invalidated immediately.`,
    'Regenerate',
    () => {
      regenerateIcTokenMutation.mutate(agent.id)
    },
    'destructive'
  )
}

function handleRevokeIcToken(agent: Agent) {
  openConfirm(
    'Revoke IC Token',
    `Revoke IC token for ${agent.name}? Agents using this token will stop working until a new one is generated.`,
    'Revoke',
    () => {
      revokeIcTokenMutation.mutate(agent.id)
    },
    'destructive'
  )
}

watch(showCreateModal, (open) => {
  if (!open) {
    name.value = ''
    selectedProviderKeyIds.value = []
    addingProviderKeyId.value = ''
    initialBudgetUsd.value = undefined
    selectedOwnerId.value = ''
  }
})

watch(showUpdateModal, (open) => {
  if (!open) resetUpdateForm()
})

function handleCopyText(text: string, label = 'Copied') {
  navigator.clipboard
    ?.writeText(text)
    ?.then(() => toast.success(label))
    ?.catch(() => toast.error('Copy failed'))
}

async function handleCopyTokenToClipboard() {
  if (!tokenDialogValue.value) return

  try {
    await navigator.clipboard?.writeText(tokenDialogValue.value)
    copyMessage.value = 'Copied to clipboard'
  } catch (_err: unknown) {
    const message = _err instanceof Error ? _err.message : 'Copy failed'
    copyMessage.value = message
  }
}
</script>

<template>
  <PageLayout title="Agents">
    <template #actions>
      <Button v-if="authStore.isAdmin" @click="showCreateModal = true">
        <IconPlus />
        Create Agent
      </Button>
    </template>

    <DataTable
      :columns="tableColumns"
      :is-loading="isLoading"
      :error="error"
      :is-empty="!agents || agents.length === 0"
      loading-text="Loading agents..."
      :on-retry="() => refetch()"
    >
      <template #empty>
        <p class="text-muted-foreground mb-4">No agents found</p>
        <Button v-if="authStore.isAdmin" @click="showCreateModal = true">
          <IconPlus />
          Create First Agent
        </Button>
      </template>

      <tr v-for="agent in agents" :key="agent.id">
        <td class="px-3 sm:px-6 py-2 max-w-[240px]">
          <button
            type="button"
            class="text-left max-w-full truncate text-base font-medium text-foreground cursor-pointer"
            :aria-label="`Copy agent name: ${agent.name}`"
            :title="agent.name"
            @click="handleCopyText(agent.name, 'Copied name')"
          >
            {{ agent.name }}
          </button>
        </td>
        <td
          v-if="showOwnerColumn"
          class="px-3 sm:px-6 py-2 text-base text-muted-foreground max-w-[220px] truncate"
          :title="ownerEmail(agent.owner_id)"
        >
          {{ ownerEmail(agent.owner_id) }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">
          <div class="flex gap-1 items-center flex-wrap max-w-[200px]">
            <span
              v-for="keyId in agent.provider_key_ids.slice(0, 3)"
              :key="keyId"
              class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted text-foreground border-border border max-w-[190px] truncate"
              :title="providerKeyLabel(keyId)"
            >
              {{ providerKeyLabel(keyId) }}
            </span>
            <Popover v-if="agent.provider_key_ids.length > 3">
              <PopoverTrigger as-child>
                <button
                  :aria-label="`Show ${agent.provider_key_ids.length - 3} more provider keys`"
                  class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted text-muted-foreground border-border border hover:text-foreground transition-colors"
                >
                  +{{ agent.provider_key_ids.length - 3 }}
                </button>
              </PopoverTrigger>
              <PopoverContent
                align="start"
                aria-label="Additional provider keys"
                class="flex flex-wrap gap-1 max-w-[320px] max-h-[200px] overflow-y-auto"
              >
                <span
                  v-for="keyId in agent.provider_key_ids.slice(3)"
                  :key="keyId"
                  class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted text-foreground border-border border"
                  :title="providerKeyLabel(keyId)"
                >
                  {{ providerKeyLabel(keyId) }}
                </span>
              </PopoverContent>
            </Popover>
          </div>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">
          <div
            v-if="icTokenStatusLoading && !getIcTokenStatus(agent.id)"
            class="text-muted-foreground"
          >
            Loading...
          </div>
          <div v-else class="flex gap-1 items-center">
            <StatusBadge
              :active="!!getIcTokenStatus(agent.id)?.has_ic_token"
              active-label="Active"
              inactive-label="None"
            />
            <div
              v-if="getIcTokenStatus(agent.id)?.created_at"
              class="text-xs text-muted-foreground max-sm:hidden"
            >
              {{ formatTimestamp(getIcTokenStatus(agent.id)?.created_at) }}
            </div>
          </div>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">
          {{ formatTimestamp(agent.created_at) }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-right text-base font-medium">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm">
                <span class="sr-only">Open menu</span>
                <IconDotsHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" class="max-w-[220px]">
              <DropdownMenuItem
                v-if="!getIcTokenStatus(agent.id)?.has_ic_token"
                :disabled="tokenActionLoadingId === agent.id"
                @click="handleGenerateIcToken(agent)"
              >
                <IconKey />
                {{ tokenActionLoadingId === agent.id ? 'Generating...' : 'Generate IC Token' }}
              </DropdownMenuItem>
              <template v-else>
                <DropdownMenuItem
                  :disabled="tokenActionLoadingId === agent.id"
                  @click="handleRegenerateIcToken(agent)"
                >
                  <IconRefresh />
                  {{
                    tokenActionLoadingId === agent.id ? 'Regenerating...' : 'Regenerate IC Token'
                  }}
                </DropdownMenuItem>
                <DropdownMenuItem
                  :disabled="tokenActionLoadingId === agent.id"
                  class="text-destructive"
                  @click="handleRevokeIcToken(agent)"
                >
                  <IconBan />
                  Revoke IC Token
                </DropdownMenuItem>
              </template>
              <template v-if="authStore.isAdmin">
                <DropdownMenuSeparator />
                <DropdownMenuItem @click="handleOpenUpdateModal(agent)">
                  <IconEdit />
                  Edit Agent
                </DropdownMenuItem>
                <DropdownMenuItem class="text-destructive" @click="handleDeleteAgent(agent)">
                  <IconTrash />
                  Delete Agent
                </DropdownMenuItem>
              </template>
            </DropdownMenuContent>
          </DropdownMenu>
        </td>
      </tr>
    </DataTable>

    <!-- Create agent modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create New Agent</DialogTitle>
          <DialogDescription>
            Create a new agent and select supported AI providers.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4">
          <div class="space-y-1.5">
            <Label for="name">Name</Label>
            <Input
              id="name"
              v-model="name"
              placeholder="My Agent"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label>Provider Keys</Label>
            <div v-if="selectedProviderKeyIds.length > 0" class="flex flex-wrap gap-1 mb-2">
              <div
                v-for="keyId in selectedProviderKeyIds"
                :key="keyId"
                class="flex items-center gap-1.5 px-2 py-1 rounded-md border border-border text-sm bg-muted max-w-[190px]"
              >
                <span class="text-xs text-foreground flex-1 truncate">{{
                  providerKeyLabel(keyId)
                }}</span>
                <button
                  type="button"
                  :aria-label="`Remove ${providerKeyLabel(keyId)}`"
                  class="ml-0.5 text-muted-foreground hover:text-destructive shrink-0"
                  @click="handleRemoveProviderKey(keyId)"
                >
                  <IconX class="h-3 w-3" />
                </button>
              </div>
            </div>
            <div v-if="availableProviderKeys.length" class="flex gap-2 max-w-full items-center">
              <Select v-model="addingProviderKeyId" :disabled="createMutation.isPending.value">
                <SelectTrigger class="flex-1 min-w-0">
                  <SelectValue placeholder="Add a provider key" />
                </SelectTrigger>
                <SelectContent class="max-w-[280px]">
                  <SelectItem
                    v-for="pk in availableProviderKeys"
                    :key="pk.id"
                    :value="String(pk.id)"
                  >
                    <span :title="pk.alias || pk.provider">{{ pk.alias || pk.provider }}</span>
                  </SelectItem>
                </SelectContent>
              </Select>
              <Button
                type="button"
                variant="outline"
                size="sm"
                class="shrink-0"
                :disabled="!addingProviderKeyId || createMutation.isPending.value"
                @click="handleAddProviderKey"
              >
                <IconPlus />
              </Button>
            </div>
          </div>

          <div class="space-y-1.5">
            <Label for="create-budget">Initial Budget (USD)</Label>
            <Input
              id="create-budget"
              v-model.number="initialBudgetUsd"
              type="number"
              min="0.01"
              step="0.01"
              placeholder="10.00"
              :disabled="createMutation.isPending.value"
            />
            <p class="text-xs text-muted-foreground">
              Required. Used to create the agent's budget.
            </p>
          </div>

          <div v-if="authStore.isAdmin" class="space-y-2">
            <Label for="create-owner">Assign to User (optional)</Label>
            <Select v-model="selectedOwnerId" :disabled="createMutation.isPending.value">
              <SelectTrigger id="create-owner">
                <SelectValue :placeholder="`Current User (${authStore.username})`" />
              </SelectTrigger>
              <SelectContent class="max-w-[280px]">
                <SelectItem v-for="user in users?.users" :key="user.id" :value="user.id">
                  <span :title="`${user.username} (${user.email || 'no email'})`"
                    >{{ user.username }} ({{ user.email || 'no email' }})</span
                  >
                </SelectItem>
              </SelectContent>
            </Select>
            <p class="text-xs text-muted-foreground">Leave empty to assign to yourself.</p>
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="
              showCreateModal = false
              name = ''
              selectedProviderKeyIds = []
              addingProviderKeyId = ''
              initialBudgetUsd = undefined
              selectedOwnerId = ''
            "
            :disabled="createMutation.isPending.value"
            variant="outline"
          >
            <IconX />
            Cancel
          </Button>
          <Button :disabled="createMutation.isPending.value" @click="handleCreateAgent">
            <IconPlus />
            {{ createMutation.isPending.value ? 'Creating...' : 'Create Agent' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Update agent modal -->
    <Dialog v-model:open="showUpdateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Edit Agent</DialogTitle>
          <DialogDescription> Update agent details and supported providers. </DialogDescription>
        </DialogHeader>

        <div class="space-y-4">
          <div class="space-y-1.5">
            <Label for="update-name">Name</Label>
            <Input
              id="update-name"
              v-model="name"
              placeholder="My Agent"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label>Provider Keys</Label>
            <div v-if="selectedProviderKeyIds.length > 0" class="flex flex-wrap gap-1 mb-2">
              <div
                v-for="keyId in selectedProviderKeyIds"
                :key="keyId"
                class="flex items-center gap-1.5 px-2 py-1 rounded-md border border-border text-sm bg-muted max-w-[190px]"
              >
                <span class="text-xs text-foreground flex-1 truncate">{{
                  providerKeyLabel(keyId)
                }}</span>
                <button
                  type="button"
                  :aria-label="`Remove ${providerKeyLabel(keyId)}`"
                  class="ml-0.5 text-muted-foreground hover:text-destructive shrink-0"
                  @click="handleRemoveProviderKey(keyId)"
                >
                  <IconX class="h-3 w-3" />
                </button>
              </div>
            </div>
            <div v-if="availableProviderKeys.length" class="flex gap-2 max-w-full items-center">
              <Select v-model="addingProviderKeyId" :disabled="updateMutation.isPending.value">
                <SelectTrigger class="flex-1 min-w-0">
                  <SelectValue placeholder="Add a provider key" />
                </SelectTrigger>
                <SelectContent class="max-w-[280px]">
                  <SelectItem
                    v-for="pk in availableProviderKeys"
                    :key="pk.id"
                    :value="String(pk.id)"
                  >
                    <span :title="pk.alias || pk.provider">{{ pk.alias || pk.provider }}</span>
                  </SelectItem>
                </SelectContent>
              </Select>
              <Button
                type="button"
                variant="outline"
                size="sm"
                class="shrink-0"
                :disabled="!addingProviderKeyId || updateMutation.isPending.value"
                @click="handleAddProviderKey"
              >
                <IconPlus />
              </Button>
            </div>
          </div>

          <div v-if="authStore.isAdmin" class="space-y-2">
            <Label for="update-owner">Owner</Label>
            <Select v-model="selectedOwnerId" :disabled="updateMutation.isPending.value">
              <SelectTrigger id="update-owner">
                <SelectValue placeholder="Select an owner" />
              </SelectTrigger>
              <SelectContent class="max-w-[280px]">
                <SelectItem v-for="user in users?.users" :key="user.id" :value="user.id">
                  <span :title="`${user.username} (${user.email || 'no email'})`"
                    >{{ user.username }} ({{ user.email || 'no email' }})</span
                  >
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <DialogFooter>
          <Button
            :disabled="updateMutation.isPending.value"
            variant="outline"
            @click="
              showUpdateModal = false
              name = ''
              selectedProviderKeyIds = []
              addingProviderKeyId = ''
              selectedOwnerId = ''
            "
          >
            <IconX />
            Cancel
          </Button>
          <Button :disabled="updateMutation.isPending.value" @click="handleUpdateAgent">
            <IconCheck />
            {{ updateMutation.isPending.value ? 'Updating...' : 'Update Agent' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- IC Token Display Modal -->
    <Dialog
      :open="showTokenDialog"
      @update:open="
        (open) => {
          showTokenDialog = open
          if (!open) {
            tokenDialogValue = ''
            tokenDialogAgentName = ''
            tokenDialogWarning = ''
            copyMessage = ''
          }
        }
      "
    >
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>IC Token for {{ tokenDialogAgentName }}</DialogTitle>
          <DialogDescription>
            Store this token securely. It is shown only once. Update your agents with this value
            immediately.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-3">
          <div
            class="rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-foreground"
          >
            <strong>Important:</strong>
            {{ tokenDialogWarning || "Copy this token now — it won't be shown again." }}
          </div>
          <div
            class="bg-muted border border-border rounded-md p-3 font-mono text-sm break-all select-all"
          >
            {{ tokenDialogValue }}
          </div>
          <p v-if="copyMessage" class="text-base text-muted-foreground">
            {{ copyMessage }}
          </p>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            @click="
              showTokenDialog = false
              tokenDialogValue = ''
              tokenDialogAgentName = ''
              tokenDialogWarning = ''
              copyMessage = ''
            "
          >
            <IconX />
            Close
          </Button>
          <Button @click="handleCopyTokenToClipboard">
            <IconCopy />
            Copy Token
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <ConfirmDialog
      v-model:open="showConfirmModal"
      :title="confirmTitle"
      :description="confirmDescription"
      :confirm-label="confirmLabel"
      :variant="confirmVariant"
      @confirm="confirmCallback?.()"
    />
  </PageLayout>
</template>
