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
import { toast } from 'vue-sonner'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { formatDate } from '@/lib/formatters'
import { useConfirm } from '@/composables/useConfirm'
import IconPlus from '@/components/icons/IconPlus.vue'
import IconX from '@/components/icons/IconX.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconDotsHorizontal from '@/components/icons/IconDotsHorizontal.vue'
import DataTable from '@/components/DataTable.vue'
import PageLayout from '@/components/PageLayout.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
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
const { showConfirmModal, confirmTitle, confirmDescription, confirmLabel, confirmVariant, confirmCallback, openConfirm } = useConfirm()
const rejectingRequest = ref<BudgetRequest | null>(null)
const rejectionReason = ref('')

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
    queryClient.invalidateQueries({ queryKey: ['budget-requests'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create budget request')
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
    queryClient.invalidateQueries({ queryKey: ['budget-requests'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to reject budget request')
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
    toast.error('Agent is required')
    return
  }

  if (createForm.value.requested_budget_usd <= 0) {
    toast.error('Budget amount must be positive')
    return
  }

  if (createForm.value.requested_budget_usd > 10000) {
    toast.error('Maximum budget increase is $10,000')
    return
  }

  if (createForm.value.justification.trim().length < 20) {
    toast.error('Justification must be at least 20 characters')
    return
  }

  if (createForm.value.justification.trim().length > 500) {
    toast.error('Justification cannot exceed 500 characters')
    return
  }

  createMutation.mutate({
    agent_id: createForm.value.agent_id,
    requester_id: authStore.username || 'default',
    requested_budget_usd: createForm.value.requested_budget_usd,
    justification: createForm.value.justification,
  })
}

function handleApproveRequest(request: BudgetRequest) {
  openConfirm(
    'Approve Budget Request',
    `Approve request #${request.id} for $${request.requested_budget_usd.toFixed(2)}?`,
    'Approve',
    () => approveMutation.mutate(request.id),
  )
}

function openRejectModal(request: BudgetRequest) {
  rejectingRequest.value = request
  rejectionReason.value = ''
  showRejectModal.value = true
}

function handleRejectRequest() {
  if (!rejectingRequest.value) return

  if (rejectionReason.value.trim().length === 0) {
    toast.error('Rejection reason is required')
    return
  }

  rejectMutation.mutate({
    requestId: rejectingRequest.value.id,
    rejection_reason: rejectionReason.value,
  })
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
        <IconPlus />
        Create Budget Request
      </Button>
    </template>

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
        <DataTable
          :columns="[
            { label: 'ID' },
            { label: 'Agent' },
            { label: 'Amount' },
            { label: 'Status' },
            { label: 'Created' },
            { label: 'Justification' },
          ]"
          :is-loading="isLoading"
          :error="error"
          :is-empty="myRequests.length === 0"
          loading-text="Loading budget requests..."
          :on-retry="() => refetch()"
        >
          <template #empty>
            <p class="text-muted-foreground mb-4">You have no budget requests yet</p>
            <Button @click="showCreateModal = true">
              <IconPlus />
              Create First Request
            </Button>
          </template>
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
        </DataTable>
      </TabsContent>

      <!-- Pending Approvals Tab (Admin Only) -->
      <TabsContent v-if="authStore.isAdmin" value="pending-approvals">
        <DataTable
          :columns="[
            { label: 'ID' },
            { label: 'Requester' },
            { label: 'Agent' },
            { label: 'Current Budget' },
            { label: 'Requested' },
            { label: 'Justification' },
            { label: 'Created' },
            { label: 'Actions', align: 'right' },
          ]"
          :is-loading="isLoading"
          :error="error"
          :is-empty="pendingRequests.length === 0"
          loading-text="Loading budget requests..."
          :on-retry="() => refetch()"
        >
          <template #empty>
            <p class="text-muted-foreground">No pending budget requests</p>
          </template>
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
                    <IconDotsHorizontal />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem @click="handleApproveRequest(request)" :disabled="approveMutation.isPending.value">
                    <IconCheck />
                    Approve
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem @click="openRejectModal(request)" :disabled="rejectMutation.isPending.value" class="text-destructive">
                    <IconX />
                    Reject
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </td>
          </tr>
        </DataTable>
      </TabsContent>

      <!-- All Requests Tab (Admin Only) -->
      <TabsContent v-if="authStore.isAdmin" value="all-requests">
        <DataTable
          :columns="[
            { label: 'ID' },
            { label: 'Requester' },
            { label: 'Agent' },
            { label: 'Amount' },
            { label: 'Status' },
            { label: 'Created' },
            { label: 'Justification' },
          ]"
          :is-loading="isLoading"
          :error="error"
          :is-empty="filteredRequests.length === 0"
          loading-text="Loading budget requests..."
          :on-retry="() => refetch()"
        >
          <template #empty>
            <p class="text-muted-foreground">No budget requests found</p>
          </template>
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
        </DataTable>
      </TabsContent>
    </Tabs>

    <!-- Create budget request modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create Budget Request</DialogTitle>
          <DialogDescription>
            Request a budget increase for an agent. Maximum request is $10,000.
          </DialogDescription>
        </DialogHeader>


        <div class="space-y-4">
          <div class="space-y-1.5">
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

          <div class="space-y-1.5">
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

          <div class="space-y-1.5">
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
            <IconX />
            Cancel
          </Button>
          <Button
            @click="handleCreateRequest"
            :disabled="createMutation.isPending.value"
          >
            <IconCheck />
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


        <div v-if="rejectingRequest" class="space-y-4">
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

          <div class="space-y-1.5">
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
            <IconX />
            Cancel
          </Button>
          <Button
            @click="handleRejectRequest"
            :disabled="rejectMutation.isPending.value"
            variant="destructive"
          >
            <IconX />
            {{ rejectMutation.isPending.value ? 'Rejecting...' : 'Reject Request' }}
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
