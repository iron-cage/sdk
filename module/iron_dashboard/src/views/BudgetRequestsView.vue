<script setup lang="ts">
import { ref, computed } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type BudgetRequest } from '../composables/useApi'
import { useAuthStore } from '../stores/auth'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
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
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui/tabs'

const api = useApi()
const authStore = useAuthStore()
const queryClient = useQueryClient()

const showCreateModal = ref(false)
const showRejectModal = ref(false)
const rejectingRequest = ref<BudgetRequest | null>(null)
const rejectionReason = ref('')
const createError = ref('')
const rejectError = ref('')

// Create request form
const createForm = ref({
  agent_id: 0,
  requested_budget_usd: 0,
  justification: '',
})

// Filter state
const statusFilter = ref<string>('all')

// Fetch budget requests
const { data: requests, isLoading, error, refetch } = useQuery({
  queryKey: ['budget-requests'],
  queryFn: () => api.listBudgetRequests(),
})

// Fetch agents for dropdown
const { data: agents } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

// Create budget request mutation
const createMutation = useMutation({
  mutationFn: (data: { agent_id: number; requester_id: string; requested_budget_usd: number; justification: string }) =>
    api.createBudgetRequest(data),
  onSuccess: () => {
    showCreateModal.value = false
    createForm.value = {
      agent_id: 0,
      requested_budget_usd: 0,
      justification: '',
    }
    createError.value = ''
    queryClient.invalidateQueries({ queryKey: ['budget-requests'] })
  },
  onError: (err) => {
    createError.value = err instanceof Error ? err.message : 'Failed to create budget request'
  },
})

// Approve budget request mutation
const approveMutation = useMutation({
  mutationFn: (requestId: string) => api.approveBudgetRequest(requestId),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['budget-requests'] })
  },
})

// Reject budget request mutation
const rejectMutation = useMutation({
  mutationFn: (data: { requestId: string; rejection_reason: string }) =>
    api.rejectBudgetRequest(data.requestId, { rejection_reason: data.rejection_reason }),
  onSuccess: () => {
    showRejectModal.value = false
    rejectingRequest.value = null
    rejectionReason.value = ''
    rejectError.value = ''
    queryClient.invalidateQueries({ queryKey: ['budget-requests'] })
  },
  onError: (err) => {
    rejectError.value = err instanceof Error ? err.message : 'Failed to reject budget request'
  },
})

// Filtered requests
const filteredRequests = computed(() => {
  if (!requests.value?.requests) return []
  if (statusFilter.value === 'all') return requests.value.requests
  return requests.value.requests.filter(r => r.status === statusFilter.value)
})

// My requests (user view)
const myRequests = computed(() => {
  if (!requests.value?.requests) return []
  return requests.value.requests.filter(r => r.requester_id === authStore.username)
})

// Pending approval requests (admin view)
const pendingRequests = computed(() => {
  if (!requests.value?.requests) return []
  return requests.value.requests.filter(r => r.status === 'pending')
})

function handleCreateRequest() {
  if (createForm.value.agent_id === 0) {
    createError.value = 'Agent is required'
    return
  }

  if (createForm.value.requested_budget_usd <= 0) {
    createError.value = 'Budget amount must be positive'
    return
  }

  if (createForm.value.requested_budget_usd > 10000) {
    createError.value = 'Maximum budget increase is $10,000'
    return
  }

  if (createForm.value.justification.trim().length < 20) {
    createError.value = 'Justification must be at least 20 characters'
    return
  }

  if (createForm.value.justification.trim().length > 500) {
    createError.value = 'Justification cannot exceed 500 characters'
    return
  }

  createError.value = ''
  createMutation.mutate({
    agent_id: createForm.value.agent_id,
    requester_id: authStore.username || 'default',
    requested_budget_usd: createForm.value.requested_budget_usd,
    justification: createForm.value.justification,
  })
}

function handleApproveRequest(request: BudgetRequest) {
  if (confirm(`Approve budget request ${request.id} for $${request.requested_budget_usd.toFixed(2)}?`)) {
    approveMutation.mutate(request.id)
  }
}

function openRejectModal(request: BudgetRequest) {
  rejectingRequest.value = request
  rejectionReason.value = ''
  rejectError.value = ''
  showRejectModal.value = true
}

function handleRejectRequest() {
  if (!rejectingRequest.value) return

  if (rejectionReason.value.trim().length === 0) {
    rejectError.value = 'Rejection reason is required'
    return
  }

  rejectMutation.mutate({
    requestId: rejectingRequest.value.id,
    rejection_reason: rejectionReason.value,
  })
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString()
}

function getStatusBadgeVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
  switch (status) {
    case 'approved':
      return 'default'
    case 'rejected':
      return 'destructive'
    case 'pending':
      return 'secondary'
    case 'cancelled':
      return 'outline'
    default:
      return 'outline'
  }
}
</script>

<template>
  <div>
    <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4 mb-6">
      <h1 class="text-xl sm:text-2xl font-bold text-gray-900">
        Budget Requests
      </h1>
      <Button @click="showCreateModal = true" class="w-full sm:w-auto">
        Create Budget Request
      </Button>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading budget requests...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading budget requests: {{ error.message }}</p>
      <Button @click="() => refetch()" variant="secondary" class="mt-4">
        Retry
      </Button>
    </div>

    <!-- Main content -->
    <div v-else>
      <Tabs default-value="my-requests" class="w-full">
        <TabsList class="mb-4">
          <TabsTrigger value="my-requests">My Requests</TabsTrigger>
          <TabsTrigger v-if="authStore.isAdmin" value="pending-approvals">
            Pending Approvals
            <Badge v-if="pendingRequests.length > 0" variant="secondary" class="ml-2">
              {{ pendingRequests.length }}
            </Badge>
          </TabsTrigger>
          <TabsTrigger v-if="authStore.isAdmin" value="all-requests">All Requests</TabsTrigger>
        </TabsList>

        <!-- My Requests Tab -->
        <TabsContent value="my-requests">
          <div v-if="myRequests.length > 0" class="bg-white rounded-lg shadow overflow-x-auto touch-pan-x">
            <table class="min-w-[700px] w-full divide-y divide-gray-200">
              <thead class="bg-gray-50">
                <tr>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    ID
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Agent
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Amount
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Created
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Justification
                  </th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200">
                <tr v-for="request in myRequests" :key="request.id">
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ request.id.substring(0, 8) }}...
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ request.agent_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    ${{ request.requested_budget_usd.toFixed(2) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap">
                    <Badge :variant="getStatusBadgeVariant(request.status)">
                      {{ request.status }}
                    </Badge>
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {{ formatDate(request.created_at) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 text-sm text-gray-500">
                    <div class="max-w-xs truncate">
                      {{ request.justification }}
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="bg-white rounded-lg shadow p-6 text-center">
            <p class="text-gray-600 mb-4">You have no budget requests yet</p>
            <Button @click="showCreateModal = true">
              Create First Request
            </Button>
          </div>
        </TabsContent>

        <!-- Pending Approvals Tab (Admin Only) -->
        <TabsContent v-if="authStore.isAdmin" value="pending-approvals">
          <div v-if="pendingRequests.length > 0" class="bg-white rounded-lg shadow overflow-x-auto touch-pan-x">
            <table class="min-w-[900px] w-full divide-y divide-gray-200">
              <thead class="bg-gray-50">
                <tr>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    ID
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Requester
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Agent
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Current Budget
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Requested
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Justification
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Created
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200">
                <tr v-for="request in pendingRequests" :key="request.id">
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ request.id.substring(0, 8) }}...
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ request.requester_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ request.agent_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    ${{ request.current_budget_usd.toFixed(2) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    ${{ request.requested_budget_usd.toFixed(2) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 text-sm text-gray-500">
                    <div class="max-w-xs">
                      {{ request.justification }}
                    </div>
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {{ formatDate(request.created_at) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                    <Button
                      @click="handleApproveRequest(request)"
                      :disabled="approveMutation.isPending.value"
                      variant="ghost"
                      size="sm"
                      class="text-green-600 hover:text-green-700"
                    >
                      Approve
                    </Button>
                    <Button
                      @click="openRejectModal(request)"
                      :disabled="rejectMutation.isPending.value"
                      variant="ghost"
                      size="sm"
                      class="text-destructive hover:text-destructive"
                    >
                      Reject
                    </Button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="bg-white rounded-lg shadow p-6 text-center">
            <p class="text-gray-600">No pending budget requests</p>
          </div>
        </TabsContent>

        <!-- All Requests Tab (Admin Only) -->
        <TabsContent v-if="authStore.isAdmin" value="all-requests">
          <div class="mb-4">
            <Label for="status-filter">Filter by Status</Label>
            <Select v-model="statusFilter">
              <SelectTrigger class="w-48">
                <SelectValue placeholder="All statuses" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All</SelectItem>
                <SelectItem value="pending">Pending</SelectItem>
                <SelectItem value="approved">Approved</SelectItem>
                <SelectItem value="rejected">Rejected</SelectItem>
                <SelectItem value="cancelled">Cancelled</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div v-if="filteredRequests.length > 0" class="bg-white rounded-lg shadow overflow-x-auto touch-pan-x">
            <table class="min-w-[800px] w-full divide-y divide-gray-200">
              <thead class="bg-gray-50">
                <tr>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    ID
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Requester
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Agent
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Amount
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Created
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Justification
                  </th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200">
                <tr v-for="request in filteredRequests" :key="request.id">
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ request.id.substring(0, 8) }}...
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ request.requester_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ request.agent_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    ${{ request.requested_budget_usd.toFixed(2) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap">
                    <Badge :variant="getStatusBadgeVariant(request.status)">
                      {{ request.status }}
                    </Badge>
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {{ formatDate(request.created_at) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 text-sm text-gray-500">
                    <div class="max-w-xs truncate">
                      {{ request.justification }}
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="bg-white rounded-lg shadow p-6 text-center">
            <p class="text-gray-600">No budget requests found</p>
          </div>
        </TabsContent>
      </Tabs>
    </div>

    <!-- Create budget request modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create Budget Request</DialogTitle>
          <DialogDescription>
            Request a budget increase for an agent. Maximum request is $10,000.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="createError" variant="destructive" class="mb-4">
          <AlertDescription>{{ createError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="agent">Agent</Label>
            <Select v-model="createForm.agent_id" :disabled="createMutation.isPending.value">
              <SelectTrigger>
                <SelectValue placeholder="Select an agent" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="agent in agents || []"
                  :key="agent.id"
                  :value="agent.id"
                >
                  {{ agent.name }} (ID: {{ agent.id }})
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-2">
            <Label for="amount">Budget Amount (USD)</Label>
            <Input
              id="amount"
              v-model.number="createForm.requested_budget_usd"
              type="number"
              min="0"
              max="10000"
              step="0.01"
              placeholder="1000.00"
              :disabled="createMutation.isPending.value"
            />
            <p class="text-xs text-gray-500">
              Maximum: $10,000
            </p>
          </div>

          <div class="space-y-2">
            <Label for="justification">Justification</Label>
            <Textarea
              id="justification"
              v-model="createForm.justification"
              placeholder="Explain why this budget increase is needed..."
              rows="4"
              :disabled="createMutation.isPending.value"
            />
            <p class="text-xs text-gray-500">
              {{ createForm.justification.length }}/500 characters (min: 20)
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
            @click="handleCreateRequest"
            :disabled="createMutation.isPending.value"
          >
            {{ createMutation.isPending.value ? 'Creating...' : 'Create Request' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Reject budget request modal -->
    <Dialog v-model:open="showRejectModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Reject Budget Request</DialogTitle>
          <DialogDescription>
            Provide a reason for rejecting this budget request.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="rejectError" variant="destructive" class="mb-4">
          <AlertDescription>{{ rejectError }}</AlertDescription>
        </Alert>

        <div v-if="rejectingRequest" class="space-y-4 py-4">
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span class="text-muted-foreground">Requester:</span>
              <span class="ml-2 font-medium">{{ rejectingRequest.requester_id }}</span>
            </div>
            <div>
              <span class="text-muted-foreground">Amount:</span>
              <span class="ml-2 font-medium">${{ rejectingRequest.requested_budget_usd.toFixed(2) }}</span>
            </div>
          </div>

          <div class="space-y-2">
            <Label for="rejection-reason">Rejection Reason (required)</Label>
            <Textarea
              id="rejection-reason"
              v-model="rejectionReason"
              placeholder="Explain why this request is being rejected..."
              rows="4"
              :disabled="rejectMutation.isPending.value"
            />
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="showRejectModal = false"
            :disabled="rejectMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="handleRejectRequest"
            :disabled="rejectMutation.isPending.value"
            variant="destructive"
          >
            {{ rejectMutation.isPending.value ? 'Rejecting...' : 'Reject Request' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
