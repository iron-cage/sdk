<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type Agent } from '../composables/useApi'
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
const name = ref('')
const selectedProviders = ref<string[]>([])
const createError = ref('')
const selectedAgent = ref<Agent | null>(null)

// Fetch agents
const { data: agents, isLoading, error, refetch } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

// Fetch providers for selection
const { data: providers } = useQuery({
  queryKey: ['providers'],
  queryFn: () => api.getProviderKeys(),
})

// Create agent mutation
const createMutation = useMutation({
  mutationFn: (data: { name: string; providers: string[] }) =>
    api.createAgent(data),
  onSuccess: () => {
    showCreateModal.value = false
    name.value = ''
    selectedProviders.value = []
    createError.value = ''
    queryClient.invalidateQueries({ queryKey: ['agents'] })
  },
  onError: (err) => {
    createError.value = err instanceof Error ? err.message : 'Failed to create agent'
  },
})

// Update agent mutation (MOCKED)
const updateMutation = useMutation({
  mutationFn: (data: { id: number; name: string; providers: string[] }) =>
    // api.updateAgent(data) // TODO: Implement updateAgent in useApi
    Promise.resolve(data), // Mock success
  onSuccess: () => {
    showUpdateModal.value = false
    selectedAgent.value = null
    name.value = ''
    selectedProviders.value = []
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
  if (!name.value || selectedProviders.value.length === 0) {
    createError.value = 'Name and at least one Provider are required'
    return
  }
  
  createError.value = ''
  createMutation.mutate({
    name: name.value,
    providers: selectedProviders.value,
  })
}

function openUpdateModal(agent: Agent) {
  selectedAgent.value = agent
  name.value = agent.name
  selectedProviders.value = [...agent.providers]
  showUpdateModal.value = true
}

function handleUpdateAgent() {
  if (!selectedAgent.value || !name.value || selectedProviders.value.length === 0) {
    createError.value = 'Name and at least one Provider are required'
    return
  }

  createError.value = ''
  updateMutation.mutate({
    id: selectedAgent.value.id,
    name: name.value,
    providers: selectedProviders.value,
  })
}

function handleDeleteAgent(agent: Agent) {
  if (confirm(`Delete agent ${agent.name}? This action cannot be undone.`)) {
    deleteMutation.mutate(agent.id)
  }
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString()
}

function toggleProvider(providerType: string) {
  if (selectedProviders.value.includes(providerType)) {
    selectedProviders.value = selectedProviders.value.filter(p => p !== providerType)
  } else {
    selectedProviders.value.push(providerType)
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
            <Label>Providers</Label>
            <div class="space-y-2 border rounded-md p-4">
              <div v-for="provider in providers" :key="provider.id" class="flex items-center space-x-2">
                <input 
                  type="checkbox"
                  :id="`create-provider-${provider.id}`" 
                  :checked="selectedProviders.includes(provider.provider)"
                  @change="toggleProvider(provider.provider)"
                  class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <Label :for="`create-provider-${provider.id}`" class="text-sm font-normal cursor-pointer">
                  {{ provider.provider }}
                </Label>
              </div>
            </div>
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
            <Label>Providers</Label>
            <div class="space-y-2 border rounded-md p-4">
              <div v-for="provider in providers" :key="provider.id" class="flex items-center space-x-2">
                <input 
                  type="checkbox"
                  :id="`update-provider-${provider.id}`" 
                  :checked="selectedProviders.includes(provider.provider)"
                  @change="toggleProvider(provider.provider)"
                  class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <Label :for="`update-provider-${provider.id}`" class="text-sm font-normal cursor-pointer">
                  {{ provider.provider }}
                </Label>
              </div>
            </div>
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
  </div>
</template>
