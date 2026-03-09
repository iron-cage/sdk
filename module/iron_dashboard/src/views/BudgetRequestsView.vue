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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import PageLayout from '@/components/PageLayout.vue'
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
  <PageLayout title="Budget Requests">
    <template #actions>
      <div v-if="authStore.isAdmin" class="w-full md:w-40">
        <Select v-model="statusFilter">
          <SelectTrigger>
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
      <Button @click="showCreateModal = true">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
        Create Budget Request
      </Button>
    </template>

    <!-- Loading state -->
    <div v-if="isLoading" class="border border-border rounded-lg p-4">
      <p class="text-muted-foreground">Loading budget requests...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="border border-border rounded-lg p-4">
      <p class="text-destructive">Error loading budget requests: {{ error.message }}</p>
      <Button @click="() => refetch()" variant="outline" class="mt-4">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
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
          <div v-if="myRequests.length > 0" class="border border-border rounded-lg overflow-x-auto touch-pan-x">
            <table class="min-w-[700px] w-full divide-y divide-border">
              <thead>
                <tr>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    ID
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Agent
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Amount
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Status
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Created
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Justification
                  </th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="request in myRequests" :key="request.id">
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    {{ request.id.substring(0, 8) }}...
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    {{ request.agent_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    ${{ request.requested_budget_usd.toFixed(2) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap">
                    <Badge :variant="getStatusBadgeVariant(request.status)">
                      {{ request.status }}
                    </Badge>
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-muted-foreground">
                    {{ formatDate(request.created_at) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 text-base text-muted-foreground">
                    <div class="max-w-xs truncate">
                      {{ request.justification }}
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="border border-border rounded-lg p-4 text-center">
            <p class="text-muted-foreground mb-4">You have no budget requests yet</p>
            <Button @click="showCreateModal = true">
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
              Create First Request
            </Button>
          </div>
        </TabsContent>

        <!-- Pending Approvals Tab (Admin Only) -->
        <TabsContent v-if="authStore.isAdmin" value="pending-approvals">
          <div v-if="pendingRequests.length > 0" class="border border-border rounded-lg overflow-x-auto touch-pan-x">
            <table class="min-w-[900px] w-full divide-y divide-border">
              <thead>
                <tr>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    ID
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Requester
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Agent
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Current Budget
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Requested
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Justification
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Created
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-right text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="request in pendingRequests" :key="request.id">
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    {{ request.id.substring(0, 8) }}...
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    {{ request.requester_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    {{ request.agent_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    ${{ request.current_budget_usd.toFixed(2) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    ${{ request.requested_budget_usd.toFixed(2) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 text-base text-muted-foreground">
                    <div class="max-w-xs">
                      {{ request.justification }}
                    </div>
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-muted-foreground">
                    {{ formatDate(request.created_at) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-right text-base font-medium">
                    <DropdownMenu>
                      <DropdownMenuTrigger as-child>
                        <Button variant="ghost" size="sm">
                          <span class="sr-only">Open menu</span>
                          <svg width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4"><path d="M3.625 7.5C3.625 8.12132 3.12132 8.625 2.5 8.625C1.87868 8.625 1.375 8.12132 1.375 7.5C1.375 6.87868 1.87868 6.375 2.5 6.375C3.12132 6.375 3.625 6.87868 3.625 7.5ZM8.625 7.5C8.625 8.12132 8.12132 8.625 7.5 8.625C6.87868 8.625 6.375 8.12132 6.375 7.5C6.375 6.87868 6.87868 6.375 7.5 6.375C8.12132 6.375 8.625 6.87868 8.625 7.5ZM13.625 7.5C13.625 8.12132 13.1213 8.625 12.5 8.625C11.8787 8.625 11.375 8.12132 11.375 7.5C11.375 6.87868 11.8787 6.375 12.5 6.375C13.1213 6.375 13.625 6.87868 13.625 7.5Z" fill="currentColor" fill-rule="evenodd" clip-rule="evenodd"></path></svg>
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem @click="handleApproveRequest(request)" :disabled="approveMutation.isPending.value">
                          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                          Approve
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem @click="openRejectModal(request)" :disabled="rejectMutation.isPending.value" class="text-destructive">
                          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                          Reject
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="border border-border rounded-lg p-4 text-center">
            <p class="text-muted-foreground">No pending budget requests</p>
          </div>
        </TabsContent>

        <!-- All Requests Tab (Admin Only) -->
        <TabsContent v-if="authStore.isAdmin" value="all-requests">
          <div v-if="filteredRequests.length > 0" class="border border-border rounded-lg overflow-x-auto touch-pan-x">
            <table class="min-w-[800px] w-full divide-y divide-border">
              <thead>
                <tr>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    ID
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Requester
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Agent
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Amount
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Status
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Created
                  </th>
                  <th class="px-3 sm:px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Justification
                  </th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="request in filteredRequests" :key="request.id">
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    {{ request.id.substring(0, 8) }}...
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    {{ request.requester_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    {{ request.agent_id }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">
                    ${{ request.requested_budget_usd.toFixed(2) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap">
                    <Badge :variant="getStatusBadgeVariant(request.status)">
                      {{ request.status }}
                    </Badge>
                  </td>
                  <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-muted-foreground">
                    {{ formatDate(request.created_at) }}
                  </td>
                  <td class="px-3 sm:px-6 py-4 text-base text-muted-foreground">
                    <div class="max-w-xs truncate">
                      {{ request.justification }}
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="border border-border rounded-lg p-4 text-center">
            <p class="text-muted-foreground">No budget requests found</p>
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
            <p class="text-xs text-muted-foreground">
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
            <p class="text-xs text-muted-foreground">
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
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="handleCreateRequest"
            :disabled="createMutation.isPending.value"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
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
          <div class="grid grid-cols-2 gap-4 text-base">
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
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="handleRejectRequest"
            :disabled="rejectMutation.isPending.value"
            variant="destructive"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            {{ rejectMutation.isPending.value ? 'Rejecting...' : 'Reject Request' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </PageLayout>
</template>
