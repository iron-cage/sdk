<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type LimitRecord, type Agent, type BudgetStatus } from '../composables/useApi'
import { useAuthStore } from '../stores/auth'
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
import { toast } from 'vue-sonner'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import PageLayout from '@/components/PageLayout.vue'
import DataTable from '@/components/DataTable.vue'
import PercentBar from '@/components/PercentBar.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const api = useApi()
const authStore = useAuthStore()
const queryClient = useQueryClient()

const showConfirmModal = ref(false)
const confirmTitle = ref('')
const confirmDescription = ref('')
const confirmLabel = ref('')
let confirmCallback: (() => void) | null = null

function openConfirm(title: string, description: string, label: string, action: () => void) {
  confirmTitle.value = title
  confirmDescription.value = description
  confirmLabel.value = label
  confirmCallback = action
  showConfirmModal.value = true
}

const showCreateModal = ref(false)
const showEditModal = ref(false)
const editingLimit = ref<LimitRecord | null>(null)
const projectId = ref('')
const overrideUserId = ref<string | null>(null)
const maxTokensPerDay = ref<number | undefined>(undefined)
const maxRequestsPerMinute = ref<number | undefined>(undefined)
const maxCostPerMonthCents = ref<number | undefined>(undefined)
const showBudgetModal = ref(false)
const budgetAgentId = ref<number | null>(null)
const budgetAgentName = ref('')
const budgetUsd = ref<number | undefined>(undefined)

// Fetch limits (hidden - global limits not integrated)
const { data: _limits, isLoading: _isLoading, error: _error, refetch: _refetch } = useQuery({
  queryKey: ['limits'],
  queryFn: () => api.getLimits(),
})
void _limits; void _isLoading; void _error; void _refetch

// Fetch agents (for owner lookup)
const { data: agents } = useQuery({
  queryKey: ['agents-for-limits'],
  queryFn: () => api.getAgents(),
})

// Fetch agent budget status
const { data: budgetStatus, isLoading: isBudgetLoading, error: budgetQueryError, refetch: refetchBudget } = useQuery({
  queryKey: ['budget-status'],
  queryFn: () => api.getBudgetStatus(),
})

// Create limit mutation
const createMutation = useMutation({
  mutationFn: ( data: { user_id: string; project_id?: string; max_tokens_per_day?: number; max_requests_per_minute?: number; max_cost_per_month_microdollars?: number } ) =>
    api.createLimit( data ),
  onSuccess: () => {
    showCreateModal.value = false
    resetForm()
    queryClient.invalidateQueries({ queryKey: ['limits'] })
  },
  onError: ( err ) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create limit')
  },
})

// Update limit mutation
const updateMutation = useMutation({
  mutationFn: ( data: { id: number; max_tokens_per_day?: number; max_requests_per_minute?: number; max_cost_per_month_microdollars?: number } ) =>
    api.updateLimit( data.id, { max_tokens_per_day: data.max_tokens_per_day, max_requests_per_minute: data.max_requests_per_minute, max_cost_per_month_microdollars: data.max_cost_per_month_microdollars } ),
  onSuccess: () => {
    showEditModal.value = false
    editingLimit.value = null
    queryClient.invalidateQueries({ queryKey: ['limits'] })
  },
  onError: ( err ) => {
    toast.error(err instanceof Error ? err.message : 'Failed to update limit')
  },
})

// Delete limit mutation
const deleteMutation = useMutation({
  mutationFn: ( id: number ) => api.deleteLimit( id ),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['limits'] })
  },
})

function resetForm() {
  projectId.value = ''
  overrideUserId.value = null
  maxTokensPerDay.value = undefined
  maxRequestsPerMinute.value = undefined
  maxCostPerMonthCents.value = undefined
}

// Convert cents to microdollars (1 cent = 10,000 microdollars)
function centsToMicrodollars( cents: number | undefined ): number | undefined {
  return cents ? cents * 10000 : undefined
}

// Convert microdollars to cents (1 cent = 10,000 microdollars)
function microdollarsToCents( microdollars: number | undefined ): number | undefined {
  return microdollars ? Math.round( microdollars / 10000 ) : undefined
}

function handleCreateLimit() {
  const userId = overrideUserId.value || authStore.username || 'default'

  // Validate at least one limit is set
  if( !maxTokensPerDay.value && !maxRequestsPerMinute.value && !maxCostPerMonthCents.value ) {
    toast.error('At least one limit must be specified')
    return
  }

  createMutation.mutate({
    user_id: userId,
    project_id: projectId.value || undefined,
    // Convert empty string/falsy to undefined (backend expects i64 or null, not "")
    max_tokens_per_day: maxTokensPerDay.value || undefined,
    max_requests_per_minute: maxRequestsPerMinute.value || undefined,
    // Convert cents (UI) to microdollars (backend)
    max_cost_per_month_microdollars: centsToMicrodollars( maxCostPerMonthCents.value ),
  })
}

function _openEditModal( limit: LimitRecord ) {
  editingLimit.value = limit
  maxTokensPerDay.value = limit.max_tokens_per_day
  maxRequestsPerMinute.value = limit.max_requests_per_minute
  // Convert microdollars (backend) to cents (UI)
  maxCostPerMonthCents.value = microdollarsToCents( limit.max_cost_per_month_microdollars )
  showEditModal.value = true
}
void _openEditModal

function handleUpdateLimit() {
  if( !editingLimit.value ) return

  // Validate at least one limit is set
  if( !maxTokensPerDay.value && !maxRequestsPerMinute.value && !maxCostPerMonthCents.value ) {
    toast.error('At least one limit must be specified')
    return
  }

  updateMutation.mutate({
    id: editingLimit.value.id,
    // Convert empty string/falsy to undefined (backend expects i64 or null, not "")
    max_tokens_per_day: maxTokensPerDay.value || undefined,
    max_requests_per_minute: maxRequestsPerMinute.value || undefined,
    // Convert cents (UI) to microdollars (backend)
    max_cost_per_month_microdollars: centsToMicrodollars( maxCostPerMonthCents.value ),
  })
}

function _handleDeleteLimit( limit: LimitRecord ) {
  openConfirm(
    'Delete Limit',
    `Delete limit ${limit.id}? This action cannot be undone.`,
    'Delete',
    () => deleteMutation.mutate( limit.id ),
  )
}
void _handleDeleteLimit

function _formatDate( timestamp: number ): string {
  return new Date( timestamp ).toLocaleString()
}
void _formatDate

function formatCost( cents: number ): string {
  return `$${( cents / 100 ).toFixed( 2 )}`
}

function findOwnerByAgentId(agentId: number): string | null {
  const match = agents?.value?.find((a: Agent) => a.id === agentId)
  return match?.owner_id || null
}

function _openCreateLimitForAgent(agentId: number) {
  const owner = findOwnerByAgentId(agentId)
  if (owner) {
    overrideUserId.value = owner
  }
  projectId.value = ''
  maxTokensPerDay.value = undefined
  maxRequestsPerMinute.value = undefined
  maxCostPerMonthCents.value = undefined
  showCreateModal.value = true
}
void _openCreateLimitForAgent

function openBudgetModal(row: BudgetStatus) {
  budgetAgentId.value = row.agent_id
  budgetAgentName.value = row.agent_name
  budgetUsd.value = Number((row.budget / 1_000_000).toFixed(2))
  showBudgetModal.value = true
}

const updateBudgetMutation = useMutation({
  mutationFn: (data: { agentId: number; total_allocated_microdollars: number }) =>
    api.updateAgentBudget(data.agentId, data.total_allocated_microdollars),
  onSuccess: () => {
    showBudgetModal.value = false
    queryClient.invalidateQueries({ queryKey: ['budget-status'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to update budget')
  },
})

function handleUpdateBudget() {
  if (!budgetAgentId.value) return
  if (!budgetUsd.value || budgetUsd.value <= 0) {
    toast.error('Budget must be greater than zero')
    return
  }

  const micros = Math.round(budgetUsd.value * 1_000_000)
  updateBudgetMutation.mutate({
    agentId: budgetAgentId.value,
    total_allocated_microdollars: micros,
  })
}
</script>

<template>
  <PageLayout title="Agent Budgets">

    <template #actions>
      <Button variant="outline" @click="refetchBudget">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
        Refresh
      </Button>
    </template>

    <!-- Agent Budgets -->
    <DataTable
      :columns="[
        { label: 'Agent' },
        { label: 'Allocated' },
        { label: 'Spent' },
        { label: 'Remaining' },
        { label: 'Used' },
        { label: 'Actions', align: 'right' },
      ]"
      :isLoading="isBudgetLoading"
      :error="budgetQueryError"
      :isEmpty="!budgetStatus?.data?.length"
      loadingText="Loading agent budgets..."
    >
      <template #empty>
        <p class="text-muted-foreground">No agent budget data available.</p>
      </template>
      <tr v-for="row in budgetStatus?.data" :key="row.agent_id">
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base font-medium text-foreground">
          {{ row.agent_name }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">
          ${{ (row.budget / 1_000_000).toFixed(2) }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">
          ${{ (row.spent / 1_000_000).toFixed(2) }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">
          ${{ (row.remaining / 1_000_000).toFixed(2) }}
        </td>
        <td class="px-3 sm:px-6 py-2 text-base text-foreground">
          <div class="flex items-center gap-2 min-w-[100px]">
            <PercentBar :percentage="row.percent_used" class="max-w-[100px]" />
            <span class="shrink-0 text-muted-foreground text-xs">{{ row.percent_used.toFixed(1) }}%</span>
          </div>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-right text-base font-medium">
          <DropdownMenu v-if="authStore.isAdmin">
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm">
                <span class="sr-only">Open menu</span>
                <svg width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4"><path d="M3.625 7.5C3.625 8.12132 3.12132 8.625 2.5 8.625C1.87868 8.625 1.375 8.12132 1.375 7.5C1.375 6.87868 1.87868 6.375 2.5 6.375C3.12132 6.375 3.625 6.87868 3.625 7.5ZM8.625 7.5C8.625 8.12132 8.12132 8.625 7.5 8.625C6.87868 8.625 6.375 8.12132 6.375 7.5C6.375 6.87868 6.87868 6.375 7.5 6.375C8.12132 6.375 8.625 6.87868 8.625 7.5ZM13.625 7.5C13.625 8.12132 13.1213 8.625 12.5 8.625C11.8787 8.625 11.375 8.12132 11.375 7.5C11.375 6.87868 11.8787 6.375 12.5 6.375C13.1213 6.375 13.625 6.87868 13.625 7.5Z" fill="currentColor" fill-rule="evenodd" clip-rule="evenodd"></path></svg>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem @click="openBudgetModal(row)">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
                Update Budget
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </td>
      </tr>
    </DataTable>

    <!-- Create limit modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create New Limit</DialogTitle>
          <DialogDescription>
            Set usage limits for tokens, requests, or cost. At least one limit must be specified.
          </DialogDescription>
        </DialogHeader>


        <div class="space-y-4">
          <div class="space-y-1.5">
            <Label for="project">Project ID (optional)</Label>
            <Input
              id="project"
              v-model="projectId"
              placeholder="my-project"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="maxTokensPerDay">Max Tokens per Day (optional)</Label>
            <Input
              id="maxTokensPerDay"
              v-model.number="maxTokensPerDay"
              type="number"
              min="1"
              placeholder="e.g., 1000000"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="maxRequestsPerMinute">Max Requests per Minute (optional)</Label>
            <Input
              id="maxRequestsPerMinute"
              v-model.number="maxRequestsPerMinute"
              type="number"
              min="1"
              placeholder="e.g., 100"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="maxCostPerMonth">Max Cost per Month in cents (optional)</Label>
            <Input
              id="maxCostPerMonth"
              v-model.number="maxCostPerMonthCents"
              type="number"
              min="1"
              placeholder="e.g., 10000 for $100.00"
              :disabled="createMutation.isPending.value"
            />
            <p v-if="maxCostPerMonthCents" class="text-base text-muted-foreground">
              = {{ formatCost( maxCostPerMonthCents ) }}/month
            </p>
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
            @click="handleCreateLimit"
            :disabled="createMutation.isPending.value"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
            {{ createMutation.isPending.value ? 'Creating...' : 'Create Limit' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Edit limit modal -->
    <Dialog v-model:open="showEditModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Edit Limit</DialogTitle>
          <DialogDescription>
            Update usage limits. At least one limit must be specified.
          </DialogDescription>
        </DialogHeader>


        <div v-if="editingLimit" class="space-y-4">
          <div class="space-y-1.5">
            <Label for="editMaxTokensPerDay">Max Tokens per Day (optional)</Label>
            <Input
              id="editMaxTokensPerDay"
              v-model.number="maxTokensPerDay"
              type="number"
              min="1"
              placeholder="e.g., 1000000"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="editMaxRequestsPerMinute">Max Requests per Minute (optional)</Label>
            <Input
              id="editMaxRequestsPerMinute"
              v-model.number="maxRequestsPerMinute"
              type="number"
              min="1"
              placeholder="e.g., 100"
              :disabled="updateMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="editMaxCostPerMonth">Max Cost per Month in cents (optional)</Label>
            <Input
              id="editMaxCostPerMonth"
              v-model.number="maxCostPerMonthCents"
              type="number"
              min="1"
              placeholder="e.g., 10000 for $100.00"
              :disabled="updateMutation.isPending.value"
            />
            <p v-if="maxCostPerMonthCents" class="text-base text-muted-foreground">
              = {{ formatCost( maxCostPerMonthCents ) }}/month
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="showEditModal = false; editingLimit = null"
            :disabled="updateMutation.isPending.value"
            variant="outline"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="handleUpdateLimit"
            :disabled="updateMutation.isPending.value"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            {{ updateMutation.isPending.value ? 'Updating...' : 'Update Limit' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Update Agent Budget Modal -->
    <Dialog v-model:open="showBudgetModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Update Agent Budget</DialogTitle>
          <DialogDescription>
            Set the total allocated budget for {{ budgetAgentName }} (in USD). Remaining will be recalculated automatically.
          </DialogDescription>
        </DialogHeader>


        <div class="space-y-4">
          <div class="space-y-1.5">
            <Label for="budget-amount">Total Budget (USD)</Label>
            <Input
              id="budget-amount"
              v-model.number="budgetUsd"
              type="number"
              min="0.01"
              step="0.01"
              placeholder="e.g., 50.00"
            />
            <p class="text-xs text-muted-foreground">
              This sets the total budget. Remaining will be total minus spent.
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showBudgetModal = false">
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button @click="handleUpdateBudget">
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            Update Budget
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
