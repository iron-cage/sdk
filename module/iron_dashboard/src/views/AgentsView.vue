<script setup lang="ts">
import { ref, watch } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type Agent, type IcTokenStatus } from '../composables/useApi'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'

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
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const api = useApi()
const queryClient = useQueryClient()
const router = useRouter()
const authStore = useAuthStore()

const showCreateModal = ref(false)
const showUpdateModal = ref(false)
const showDeleteModal = ref(false)
const name = ref('')
const selectedProviderKeyId = ref<number | null>(null)
const initialBudgetUsd = ref<number | null>(null)
const createError = ref('')
const selectedAgent = ref<Agent | null>(null)
const agentToDelete = ref<Agent | null>(null)
const icTokenStatuses = ref<Record<number, IcTokenStatus>>({})
const icTokenStatusLoading = ref(false)
const icTokenError = ref('')
const tokenActionLoadingId = ref<number | null>(null)
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
    icTokenError.value = ''
    const statusMap: Record<number, IcTokenStatus> = {}

    await Promise.all(
      agentList.map(async (agent) => {
        try {
          const status = await api.getIcTokenStatus(agent.id)
          statusMap[agent.id] = status
        } catch (err) {
          if (!icTokenError.value) {
            icTokenError.value = err instanceof Error ? err.message : 'Failed to load IC token status'
          }
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

// Create agent mutation
const createMutation = useMutation({
  mutationFn: (data: { name: string; providers: string[]; provider_key_id: number; initial_budget_microdollars: number }) =>
    api.createAgent(data),
  onSuccess: () => {
    showCreateModal.value = false
    name.value = ''
    selectedProviderKeyId.value = null
    initialBudgetUsd.value = null
    createError.value = ''
    queryClient.invalidateQueries({ queryKey: ['agents'] })
  },
  onError: (err) => {
    createError.value = err instanceof Error ? err.message : 'Failed to create agent'
  },
})

// Update agent mutation
const updateMutation = useMutation({
  mutationFn: (data: { id: number; name: string; providers: string[]; provider_key_id?: number | null }) =>
    api.updateAgent(data),
  onSuccess: () => {
    showUpdateModal.value = false
    selectedAgent.value = null
    name.value = ''
    selectedProviderKeyId.value = null
    createError.value = ''
    queryClient.invalidateQueries({ queryKey: ['agents'] })
  },
  onError: (err) => {
    createError.value = err instanceof Error ? err.message : 'Failed to update agent'
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
  if (!name.value) {
    createError.value = 'Name is required'
    return
  }

  if (selectedProviderKeyId.value === null) {
    createError.value = 'Provider key is required'
    return
  }

  if (!initialBudgetUsd.value || initialBudgetUsd.value <= 0) {
    createError.value = 'Initial budget (USD) is required and must be positive'
    return
  }

  const providerKeyId = Number(selectedProviderKeyId.value)
  const providerRecord = providers.value?.find(p => p.id === providerKeyId)
  if (!providerRecord) {
    createError.value = 'Selected provider key not found'
    return
  }

  const budgetMicros = Math.round(initialBudgetUsd.value * 1_000_000)

  createError.value = ''
  createMutation.mutate({
    name: name.value,
    providers: [providerRecord.provider],
    provider_key_id: providerKeyId,
    initial_budget_microdollars: budgetMicros,
  })
}

function openUpdateModal(agent: Agent) {
  selectedAgent.value = agent
  name.value = agent.name
  selectedProviderKeyId.value = agent.provider_key_id ?? null
  showUpdateModal.value = true
}

function handleUpdateAgent() {
  if (!selectedAgent.value || !name.value) {
    createError.value = 'Name is required'
    return
  }

  if (selectedProviderKeyId.value === null) {
    createError.value = 'Provider key is required'
    return
  }

  const providerKeyId = Number(selectedProviderKeyId.value)
  const providerRecord = providers.value?.find(p => p.id === providerKeyId)
  if (!providerRecord) {
    createError.value = 'Selected provider key not found'
    return
  }

  createError.value = ''
  updateMutation.mutate({
    id: selectedAgent.value.id,
    name: name.value,
    providers: [providerRecord.provider],
    provider_key_id: providerKeyId,
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

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString()
}

function formatTimestamp(timestamp?: number | null): string {
  if (!timestamp) return '-'
  // IC token timestamps are seconds; agent created_at is milliseconds. Normalize to ms for display.
  const millis = timestamp > 1_000_000_000_000 ? timestamp : timestamp * 1000
  return new Date(millis).toLocaleString()
}

function getIcTokenStatus(agentId: number): IcTokenStatus | undefined {
  return icTokenStatuses.value[agentId]
}

async function handleGenerateIcToken(agent: Agent) {
  tokenActionLoadingId.value = agent.id
  icTokenError.value = ''
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
    icTokenError.value = err instanceof Error ? err.message : 'Failed to generate IC token'
  } finally {
    tokenActionLoadingId.value = null
  }
}

async function handleRegenerateIcToken(agent: Agent) {
  if (!confirm(`Regenerate IC token for ${agent.name}? This will invalidate the current token.`)) {
    return
  }

  tokenActionLoadingId.value = agent.id
  icTokenError.value = ''
  try {
    const response = await api.regenerateIcToken(agent.id)
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
    tokenDialogWarning.value = response.warning || 'Old IC token is now invalid.'
    copyMessage.value = ''
    showTokenDialog.value = true
  } catch (err) {
    icTokenError.value = err instanceof Error ? err.message : 'Failed to regenerate IC token'
  } finally {
    tokenActionLoadingId.value = null
  }
}

async function handleRevokeIcToken(agent: Agent) {
  if (!confirm(`Revoke IC token for ${agent.name}? Agents using this token will stop working until a new one is generated.`)) {
    return
  }

  tokenActionLoadingId.value = agent.id
  icTokenError.value = ''
  try {
    await api.revokeIcToken(agent.id)
    icTokenStatuses.value = {
      ...icTokenStatuses.value,
      [agent.id]: {
        agent_id: agent.id,
        has_ic_token: false,
        created_at: null,
      },
    }
  } catch (err) {
    icTokenError.value = err instanceof Error ? err.message : 'Failed to revoke IC token'
  } finally {
    tokenActionLoadingId.value = null
  }
}

async function copyTokenToClipboard() {
  if (!tokenDialogValue.value) return

  try {
    await navigator.clipboard.writeText(tokenDialogValue.value)
    copyMessage.value = 'Copied to clipboard'
  } catch (err) {
    copyMessage.value = err instanceof Error ? err.message : 'Copy failed'
  }
}
</script>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-gray-900">Agents</h1>
      <Button v-if="authStore.isAdmin" @click="showCreateModal = true">
        Create Agent
      </Button>
    </div>

    <Alert v-if="icTokenError" variant="destructive" class="mb-4">
      <AlertDescription>{{ icTokenError }}</AlertDescription>
    </Alert>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading agents...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading agents: {{ error.message }}</p>
      <Button @click="() => refetch()" variant="secondary" class="mt-4">
        Retry
      </Button>
    </div>

    <!-- Agents table -->
    <div v-else-if="agents && agents.length > 0" class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Name
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Providers
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Provider Key
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              IC Token
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
          <tr v-for="agent in agents" :key="agent.id">
            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
              {{ agent.name }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              <div class="flex gap-1 flex-wrap">
                <span 
                  v-for="provider in agent.providers" 
                  :key="provider"
                  class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800"
                >
                  {{ provider }}
                </span>
              </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ agent.provider_key_id ?? 'None' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              <div v-if="icTokenStatusLoading && !getIcTokenStatus(agent.id)" class="text-gray-500">
                Loading...
              </div>
              <div v-else>
                <Badge
                  v-if="getIcTokenStatus(agent.id)?.has_ic_token"
                  variant="default"
                >
                  Active
                </Badge>
                <Badge
                  v-else
                  variant="outline"
                  class="text-gray-700"
                >
                  None
                </Badge>
                <div
                  v-if="getIcTokenStatus(agent.id)?.created_at"
                  class="text-xs text-gray-500 mt-1"
                >
                  Created {{ formatTimestamp(getIcTokenStatus(agent.id)?.created_at) }}
                </div>
              </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ formatDate(agent.created_at) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
              <DropdownMenu>
                <DropdownMenuTrigger as-child>
                  <Button variant="ghost" size="sm">
                    <span class="sr-only">Open menu</span>
                    <svg width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4"><path d="M3.625 7.5C3.625 8.12132 3.12132 8.625 2.5 8.625C1.87868 8.625 1.375 8.12132 1.375 7.5C1.375 6.87868 1.87868 6.375 2.5 6.375C3.12132 6.375 3.625 6.87868 3.625 7.5ZM8.625 7.5C8.625 8.12132 8.12132 8.625 7.5 8.625C6.87868 8.625 6.375 8.12132 6.375 7.5C6.375 6.87868 6.87868 6.375 7.5 6.375C8.12132 6.375 8.625 6.87868 8.625 7.5ZM13.625 7.5C13.625 8.12132 13.1213 8.625 12.5 8.625C11.8787 8.625 11.375 8.12132 11.375 7.5C11.375 6.87868 11.8787 6.375 12.5 6.375C13.1213 6.375 13.625 6.87868 13.625 7.5Z" fill="currentColor" fill-rule="evenodd" clip-rule="evenodd"></path></svg>
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuLabel>Actions</DropdownMenuLabel>
                  <DropdownMenuItem @click="router.push(`/agents/${agent.id}/tokens`)">
                    Manage Tokens
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    v-if="!getIcTokenStatus(agent.id)?.has_ic_token"
                    @click="handleGenerateIcToken(agent)"
                    :disabled="tokenActionLoadingId === agent.id"
                  >
                    {{ tokenActionLoadingId === agent.id ? 'Generating...' : 'Generate IC Token' }}
                  </DropdownMenuItem>
                  <template v-else>
                    <DropdownMenuItem
                      @click="handleRegenerateIcToken(agent)"
                      :disabled="tokenActionLoadingId === agent.id"
                    >
                      {{ tokenActionLoadingId === agent.id ? 'Regenerating...' : 'Regenerate IC Token' }}
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      @click="handleRevokeIcToken(agent)"
                      :disabled="tokenActionLoadingId === agent.id"
                      class="text-red-600"
                    >
                      Revoke IC Token
                    </DropdownMenuItem>
                  </template>
                  <template v-if="authStore.isAdmin">
                    <DropdownMenuSeparator />
                    <DropdownMenuItem @click="openUpdateModal(agent)">
                      Edit Agent
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="handleDeleteAgent(agent)" class="text-red-600">
                      Delete Agent
                    </DropdownMenuItem>
                  </template>
                </DropdownMenuContent>
              </DropdownMenu>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else class="bg-white rounded-lg shadow p-6 text-center">
      <p class="text-gray-600 mb-4">No agents found</p>
      <Button v-if="authStore.isAdmin" @click="showCreateModal = true">
        Create First Agent
      </Button>
    </div>

    <!-- Create agent modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create New Agent</DialogTitle>
          <DialogDescription>
            Create a new agent and select supported AI providers.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="createError" variant="destructive" class="mb-4">
          <AlertDescription>{{ createError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="name">Name</Label>
            <Input
              id="name"
              v-model="name"
              placeholder="My Agent"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-2">
            <Label for="create-provider-key">Assigned Provider Key (required)</Label>
            <select
              id="create-provider-key"
              v-model="selectedProviderKeyId"
              :disabled="createMutation.isPending.value"
              class="w-full border rounded-md px-3 py-2 text-sm"
            >
              <option :value="null">None</option>
              <option
                v-for="providerKey in providers"
                :key="providerKey.id"
                :value="providerKey.id"
              >
                {{ providerKey.id }} - {{ providerKey.provider }}
              </option>
            </select>
          </div>

          <div class="space-y-2">
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
            <p class="text-xs text-gray-500">
              Required. Used to create the agent's budget (microdollars on backend).
            </p>
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
            @click="handleCreateAgent"
            :disabled="createMutation.isPending.value"
          >
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

        <Alert v-if="createError" variant="destructive" class="mb-4">
          <AlertDescription>{{ createError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="update-name">Name</Label>
            <Input
              id="update-name"
              v-model="name"
              placeholder="My Agent"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="space-y-2">
            <Label for="update-provider-key">Assigned Provider Key</Label>
            <select
              id="update-provider-key"
              v-model="selectedProviderKeyId"
              :disabled="updateMutation.isPending.value"
              class="w-full border rounded-md px-3 py-2 text-sm"
            >
              <option :value="null">None</option>
              <option
                v-for="providerKey in providers"
                :key="providerKey.id"
                :value="providerKey.id"
              >
                {{ providerKey.id }} - {{ providerKey.provider }}
              </option>
            </select>
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="showUpdateModal = false"
            :disabled="updateMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="handleUpdateAgent"
            :disabled="updateMutation.isPending.value"
          >
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
            @click="showDeleteModal = false"
            :disabled="deleteMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="confirmDelete"
            :disabled="deleteMutation.isPending.value"
            variant="destructive"
          >
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
          <div class="bg-gray-100 border rounded-md p-3 font-mono text-sm break-all">
            {{ tokenDialogValue }}
          </div>
          <p class="text-sm text-yellow-700">
            {{ tokenDialogWarning }}
          </p>
          <p class="text-xs text-gray-500">
            After closing this dialog you will not be able to view the token again. Regenerate if you need a new value.
          </p>
          <p v-if="copyMessage" class="text-sm text-gray-600">
            {{ copyMessage }}
          </p>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showTokenDialog = false">
            Close
          </Button>
          <Button @click="copyTokenToClipboard">
            Copy Token
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
