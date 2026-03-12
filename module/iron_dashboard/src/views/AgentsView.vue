<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type Agent, type IcTokenStatus } from '../composables/useApi'
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
import { toast } from 'vue-sonner'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useAuthStore } from '../stores/auth'
import { formatDate, formatTimestamp } from '@/lib/formatters'
import { useConfirm } from '@/composables/useConfirm'
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

const api = useApi()
const queryClient = useQueryClient()
const authStore = useAuthStore()

const showCreateModal = ref(false)
const showUpdateModal = ref(false)
const showDeleteModal = ref(false)
const { showConfirmModal, confirmTitle, confirmDescription, confirmLabel, confirmCallback, openConfirm } = useConfirm()
const name = ref('')
const selectedProviderKeyIds = ref<number[]>([])
const addingProviderKeyId = ref<string>('')
const initialBudgetUsd = ref<number | undefined>(undefined)
const selectedOwnerId = ref<string>('')
const selectedAgent = ref<Agent | null>(null)
const agentToDelete = ref<Agent | null>(null)
const icTokenStatuses = ref<Record<number, IcTokenStatus>>({})
const icTokenStatusLoading = ref(false)
const tokenActionLoadingId = ref<number | null>(null)
const createFormError = ref('')
const updateFormError = ref('')
const showTokenDialog = ref(false)
const tokenDialogValue = ref('')
const tokenDialogAgentName = ref('')
const tokenDialogWarning = ref('')
const copyMessage = ref('')

// Fetch agents
const { data: agents, isLoading, error, refetch } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

// Fetch IC token status for each agent once agents are loaded
watch(
  () => agents?.value,
  async (agentList) => {
    if (!agentList) {
      icTokenStatuses.value = {}
      return
    }

    icTokenStatusLoading.value = true
    const statusMap: Record<number, IcTokenStatus> = {}

    await Promise.all(
      agentList.map(async (agent) => {
        try {
          const status = await api.getIcTokenStatus(agent.id)
          statusMap[agent.id] = status
        } catch {
          toast.error('Failed to load IC token status for agent ' + agent.name)
        }
      })
    )

    icTokenStatuses.value = statusMap
    icTokenStatusLoading.value = false
  },
  { immediate: true }
)

// Fetch providers for selection
const { data: providers } = useQuery({
  queryKey: ['providers'],
  queryFn: () => api.getProviderKeys(),
})

const availableProviderKeys = computed(() =>
  (providers.value ?? []).filter(p => !selectedProviderKeyIds.value.includes(p.id))
)

function addProviderKey() {
  const id = Number(addingProviderKeyId.value)
  if (id && !selectedProviderKeyIds.value.includes(id)) {
    selectedProviderKeyIds.value = [...selectedProviderKeyIds.value, id]
    addingProviderKeyId.value = ''
  }
}

function removeProviderKey(keyId: number) {
  selectedProviderKeyIds.value = selectedProviderKeyIds.value.filter(id => id !== keyId)
}

function providerKeyLabel(keyId: number): string {
  const key = providers.value?.find(p => p.id === keyId)
  if (!key) return `#${keyId}`
  return key.alias || key.provider
}

// Fetch users for owner selection (admin only)
const { data: users } = useQuery({
  queryKey: ['users-for-agents'],
  queryFn: () => api.getUsers({ is_active: true }),
  enabled: authStore.isAdmin,
})

// Create agent mutation
const createMutation = useMutation({
  mutationFn: (data: { name: string; providers: string[]; provider_key_ids: number[]; initial_budget_microdollars: number; owner_id?: string }) =>
    api.createAgent(data),
  onSuccess: () => {
    showCreateModal.value = false
    name.value = ''
    selectedProviderKeyIds.value = []
    addingProviderKeyId.value = ''
    initialBudgetUsd.value = undefined
    selectedOwnerId.value = ''
    createFormError.value = ''
    queryClient.invalidateQueries({ queryKey: ['agents'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create agent')
  },
})

// Update agent mutation
const updateMutation = useMutation({
  mutationFn: (data: { id: number; name: string; providers: string[]; provider_key_ids: number[]; owner_id?: string }) =>
    api.updateAgent(data),
  onSuccess: () => {
    showUpdateModal.value = false
    selectedAgent.value = null
    name.value = ''
    selectedProviderKeyIds.value = []
    addingProviderKeyId.value = ''
    selectedOwnerId.value = ''
    updateFormError.value = ''
    queryClient.invalidateQueries({ queryKey: ['agents'] })
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
})

function handleCreateAgent() {
  createFormError.value = ''
  if (!name.value) {
    createFormError.value = 'Name is required'
    return
  }

  if (selectedProviderKeyIds.value.length === 0) {
    createFormError.value = 'At least one provider key is required'
    return
  }

  if (!initialBudgetUsd.value || initialBudgetUsd.value <= 0) {
    createFormError.value = 'Initial budget (USD) is required and must be positive'
    return
  }

  const selectedKeys = (providers.value ?? []).filter(p => selectedProviderKeyIds.value.includes(p.id))
  const uniqueProviders = [...new Set(selectedKeys.map(p => p.provider))]
  const budgetMicros = Math.round(initialBudgetUsd.value * 1_000_000)

  createMutation.mutate({
    name: name.value,
    providers: uniqueProviders,
    provider_key_ids: selectedProviderKeyIds.value,
    initial_budget_microdollars: budgetMicros,
    owner_id: selectedOwnerId.value || undefined,
  })
}

function openUpdateModal(agent: Agent) {
  selectedAgent.value = agent
  name.value = agent.name
  selectedProviderKeyIds.value = [...(agent.provider_key_ids ?? [])]
  addingProviderKeyId.value = ''
  selectedOwnerId.value = agent.owner_id ?? ''
  updateFormError.value = ''
  showUpdateModal.value = true
}

function handleUpdateAgent() {
  updateFormError.value = ''
  if (!selectedAgent.value || !name.value) {
    updateFormError.value = 'Name is required'
    return
  }

  if (selectedProviderKeyIds.value.length === 0) {
    updateFormError.value = 'At least one provider key is required'
    return
  }

  const selectedKeys = (providers.value ?? []).filter(p => selectedProviderKeyIds.value.includes(p.id))
  const uniqueProviders = [...new Set(selectedKeys.map(p => p.provider))]

  updateMutation.mutate({
    id: selectedAgent.value.id,
    name: name.value,
    providers: uniqueProviders,
    provider_key_ids: selectedProviderKeyIds.value,
    owner_id: selectedOwnerId.value || undefined,
  })
}

function handleDeleteAgent(agent: Agent) {
  agentToDelete.value = agent
  showDeleteModal.value = true
}

function confirmDelete() {
  if (agentToDelete.value) {
    deleteMutation.mutate(agentToDelete.value.id)
    showDeleteModal.value = false
    agentToDelete.value = null
  }
}

function getIcTokenStatus(agentId: number): IcTokenStatus | undefined {
  return icTokenStatuses.value[agentId]
}

async function handleGenerateIcToken(agent: Agent) {
  tokenActionLoadingId.value = agent.id
  try {
    const response = await api.generateIcToken(agent.id)
    icTokenStatuses.value = {
      ...icTokenStatuses.value,
      [agent.id]: {
        agent_id: agent.id,
        has_ic_token: true,
        created_at: response.created_at,
      },
    }
    tokenDialogAgentName.value = agent.name
    tokenDialogValue.value = response.ic_token
    tokenDialogWarning.value = response.warning
    copyMessage.value = ''
    showTokenDialog.value = true
  } catch (err) {
    toast.error(err instanceof Error ? err.message : 'Failed to generate IC token')
  } finally {
    tokenActionLoadingId.value = null
  }
}

async function handleRegenerateIcToken(agent: Agent) {
  openConfirm(
    'Regenerate IC Token',
    `Regenerate IC token for ${agent.name}? The current token will be invalidated immediately.`,
    'Regenerate',
    async () => {
      tokenActionLoadingId.value = agent.id
      try {
        const response = await api.regenerateIcToken(agent.id)
        icTokenStatuses.value = {
          ...icTokenStatuses.value,
          [agent.id]: { agent_id: agent.id, has_ic_token: true, created_at: response.created_at },
        }
        tokenDialogAgentName.value = agent.name
        tokenDialogValue.value = response.ic_token
        tokenDialogWarning.value = response.warning || 'Old IC token is now invalid.'
        copyMessage.value = ''
        showTokenDialog.value = true
      } catch (err) {
        toast.error(err instanceof Error ? err.message : 'Failed to regenerate IC token')
      } finally {
        tokenActionLoadingId.value = null
      }
    },
  )
}

async function handleRevokeIcToken(agent: Agent) {
  openConfirm(
    'Revoke IC Token',
    `Revoke IC token for ${agent.name}? Agents using this token will stop working until a new one is generated.`,
    'Revoke',
    async () => {
      tokenActionLoadingId.value = agent.id
      try {
        await api.revokeIcToken(agent.id)
        icTokenStatuses.value = {
          ...icTokenStatuses.value,
          [agent.id]: { agent_id: agent.id, has_ic_token: false, created_at: null },
        }
      } catch (err) {
        toast.error(err instanceof Error ? err.message : 'Failed to revoke IC token')
      } finally {
        tokenActionLoadingId.value = null
      }
    },
  )
}

async function copyTokenToClipboard() {
  if (!tokenDialogValue.value) return

  try {
    await navigator.clipboard.writeText(tokenDialogValue.value)
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
      :columns="[
        { label: 'Name' },
        { label: 'Owner' },
        { label: 'Providers' },
        { label: 'IC Token' },
        { label: 'Created' },
        { label: 'Actions', align: 'right' },
      ]"
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
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base font-medium text-foreground">
          {{ agent.name }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground max-w-[200px] truncate">
          {{ agent.owner_id || 'Unknown' }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">
          <div class="flex gap-1 flex-wrap items-center">
            <span
              v-for="keyId in agent.provider_key_ids"
              :key="keyId"
              class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted text-foreground border-border border"
            >
              {{ providerKeyLabel(keyId) }}
            </span>
          </div>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">
          <div v-if="icTokenStatusLoading && !getIcTokenStatus(agent.id)" class="text-muted-foreground">
            Loading...
          </div>
          <div v-else class="flex gap-1 items-center">
            <StatusBadge :active="!!getIcTokenStatus(agent.id)?.has_ic_token" active-label="Active" inactive-label="None" />
            <div v-if="getIcTokenStatus(agent.id)?.created_at" class="text-xs text-muted-foreground">
              {{ formatTimestamp(getIcTokenStatus(agent.id)?.created_at) }}
            </div>
          </div>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">
          {{ formatDate(agent.created_at) }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-right text-base font-medium">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm">
                <span class="sr-only">Open menu</span>
                <IconDotsHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
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
                  {{ tokenActionLoadingId === agent.id ? 'Regenerating...' : 'Regenerate IC Token' }}
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
                <DropdownMenuItem @click="openUpdateModal(agent)">
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
                class="flex items-center gap-1.5 px-2 py-1 rounded-md border border-border text-sm bg-muted"
              >
                <span class="text-xs text-foreground">{{ providerKeyLabel(keyId) }}</span>
                <button
                  type="button"
                  class="ml-0.5 text-muted-foreground hover:text-destructive"
                  @click="removeProviderKey(keyId)"
                >
                  <IconX class="h-3 w-3" />
                </button>
              </div>
            </div>
            <div v-if="availableProviderKeys.length" class="flex gap-2">
              <Select v-model="addingProviderKeyId" :disabled="createMutation.isPending.value">
                <SelectTrigger class="flex-1">
                  <SelectValue placeholder="Add a provider key" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="pk in availableProviderKeys"
                    :key="pk.id"
                    :value="String(pk.id)"
                  >
                    {{ pk.alias || pk.provider }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <Button type="button" variant="outline" size="sm" :disabled="!addingProviderKeyId || createMutation.isPending.value" @click="addProviderKey">
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
              <SelectContent>
                <SelectItem
                  v-for="user in users?.users"
                  :key="user.id"
                  :value="user.id"
                >
                  {{ user.username }} ({{ user.email || 'no email' }})
                </SelectItem>
              </SelectContent>
            </Select>
            <p class="text-xs text-muted-foreground">
              Leave empty to assign to yourself.
            </p>
          </div>
        </div>

        <p v-if="createFormError" class="text-sm text-destructive">{{ createFormError }}</p>

        <DialogFooter>
          <Button
            :disabled="createMutation.isPending.value"
            variant="outline"
            @click="showCreateModal = false; createFormError = ''"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="createMutation.isPending.value"
            @click="handleCreateAgent"
          >
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
          <DialogDescription>
            Update agent details and supported providers.
          </DialogDescription>
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
                class="flex items-center gap-1.5 px-2 py-1 rounded-md border border-border text-sm bg-muted"
              >
                <span class="text-xs text-foreground">{{ providerKeyLabel(keyId) }}</span>
                <button
                  type="button"
                  class="ml-0.5 text-muted-foreground hover:text-destructive"
                  @click="removeProviderKey(keyId)"
                >
                  <IconX class="h-3 w-3" />
                </button>
              </div>
            </div>
            <div v-if="availableProviderKeys.length" class="flex gap-2">
              <Select v-model="addingProviderKeyId" :disabled="updateMutation.isPending.value">
                <SelectTrigger class="flex-1">
                  <SelectValue placeholder="Add a provider key" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="pk in availableProviderKeys"
                    :key="pk.id"
                    :value="String(pk.id)"
                  >
                    {{ pk.alias || pk.provider }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <Button type="button" variant="outline" size="sm" :disabled="!addingProviderKeyId || updateMutation.isPending.value" @click="addProviderKey">
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
              <SelectContent>
                <SelectItem
                  v-for="user in users?.users"
                  :key="user.id"
                  :value="user.id"
                >
                  {{ user.username }} ({{ user.email || 'no email' }})
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <p v-if="updateFormError" class="text-sm text-destructive">{{ updateFormError }}</p>

        <DialogFooter>
          <Button
            :disabled="updateMutation.isPending.value"
            variant="outline"
            @click="showUpdateModal = false; updateFormError = ''"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="updateMutation.isPending.value"
            @click="handleUpdateAgent"
          >
            <IconCheck />
            {{ updateMutation.isPending.value ? 'Updating...' : 'Update Agent' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Delete Confirmation Modal -->
    <Dialog v-model:open="showDeleteModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Delete Agent</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{{ agentToDelete?.name }}"? This action cannot be undone.
          </DialogDescription>
        </DialogHeader>

        <DialogFooter>
          <Button
            :disabled="deleteMutation.isPending.value"
            variant="outline"
            @click="showDeleteModal = false"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="deleteMutation.isPending.value"
            variant="destructive"
            @click="confirmDelete"
          >
            <IconTrash />
            {{ deleteMutation.isPending.value ? 'Deleting...' : 'Delete' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- IC Token Display Modal -->
    <Dialog v-model:open="showTokenDialog">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>IC Token for {{ tokenDialogAgentName }}</DialogTitle>
          <DialogDescription>
            Store this token securely. It is shown only once. Update your agents with this value immediately.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-3">
          <div class="rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-foreground">
            <strong>Important:</strong> {{ tokenDialogWarning || 'Copy this token now — it won\'t be shown again.' }}
          </div>
          <div class="bg-muted border border-border rounded-md p-3 font-mono text-sm break-all select-all">
            {{ tokenDialogValue }}
          </div>
          <p v-if="copyMessage" class="text-base text-muted-foreground">
            {{ copyMessage }}
          </p>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showTokenDialog = false; tokenDialogValue = ''">
            <IconX />
            Close
          </Button>
          <Button @click="copyTokenToClipboard">
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
      @confirm="confirmCallback?.()"
    />
  </PageLayout>
</template>
