<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type LimitRecord } from '../composables/useApi'
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
import { Alert, AlertDescription } from '@/components/ui/alert'

const api = useApi()
const authStore = useAuthStore()
const queryClient = useQueryClient()

const showCreateModal = ref(false)
const showEditModal = ref(false)
const editingLimit = ref<LimitRecord | null>(null)
const projectId = ref('')
const maxTokensPerDay = ref<number | undefined>(undefined)
const maxRequestsPerMinute = ref<number | undefined>(undefined)
const maxCostPerMonthCents = ref<number | undefined>(undefined)
const createError = ref('')
const editError = ref('')

// Fetch limits
const { data: limits, isLoading, error, refetch } = useQuery({
  queryKey: ['limits'],
  queryFn: () => api.getLimits(),
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
    createError.value = err instanceof Error ? err.message : 'Failed to create limit'
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
    editError.value = err instanceof Error ? err.message : 'Failed to update limit'
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
  maxTokensPerDay.value = undefined
  maxRequestsPerMinute.value = undefined
  maxCostPerMonthCents.value = undefined
  createError.value = ''
  editError.value = ''
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
  createError.value = ''

  // Validate at least one limit is set
  if( !maxTokensPerDay.value && !maxRequestsPerMinute.value && !maxCostPerMonthCents.value ) {
    createError.value = 'At least one limit must be specified'
    return
  }

  createMutation.mutate({
    user_id: authStore.username || 'default',
    project_id: projectId.value || undefined,
    // Convert empty string/falsy to undefined (backend expects i64 or null, not "")
    max_tokens_per_day: maxTokensPerDay.value || undefined,
    max_requests_per_minute: maxRequestsPerMinute.value || undefined,
    // Convert cents (UI) to microdollars (backend)
    max_cost_per_month_microdollars: centsToMicrodollars( maxCostPerMonthCents.value ),
  })
}

function openEditModal( limit: LimitRecord ) {
  editingLimit.value = limit
  maxTokensPerDay.value = limit.max_tokens_per_day
  maxRequestsPerMinute.value = limit.max_requests_per_minute
  // Convert microdollars (backend) to cents (UI)
  maxCostPerMonthCents.value = microdollarsToCents( limit.max_cost_per_month_microdollars )
  editError.value = ''
  showEditModal.value = true
}

function handleUpdateLimit() {
  if( !editingLimit.value ) return
  editError.value = ''

  // Validate at least one limit is set
  if( !maxTokensPerDay.value && !maxRequestsPerMinute.value && !maxCostPerMonthCents.value ) {
    editError.value = 'At least one limit must be specified'
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

function handleDeleteLimit( limit: LimitRecord ) {
  if( confirm( `Delete limit ${limit.id}? This action cannot be undone.` ) ) {
    deleteMutation.mutate( limit.id )
  }
}

function formatDate( timestamp: number ): string {
  return new Date( timestamp ).toLocaleString()
}

function formatCost( cents: number ): string {
  return `$${( cents / 100 ).toFixed( 2 )}`
}
</script>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-gray-900">Usage Limits</h1>
      <Button @click="showCreateModal = true">
        Create New Limit
      </Button>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading limits...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading limits: {{ error.message }}</p>
      <Button @click="() => refetch()" variant="secondary" class="mt-4">
        Retry
      </Button>
    </div>

    <!-- Limits table -->
    <div v-else-if="limits && limits.length > 0" class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Project
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Tokens/Day
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Requests/Min
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Cost/Month
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
          <tr v-for="limit in limits" :key="limit.id">
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ limit.id }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ limit.project_id || '-' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ limit.max_tokens_per_day?.toLocaleString() || '-' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ limit.max_requests_per_minute?.toLocaleString() || '-' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
              {{ limit.max_cost_per_month_microdollars ? formatCost( microdollarsToCents( limit.max_cost_per_month_microdollars ) || 0 ) : '-' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ formatDate( limit.created_at ) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
              <Button
                @click="openEditModal( limit )"
                :disabled="updateMutation.isPending.value"
                variant="ghost"
                size="sm"
              >
                Edit
              </Button>
              <Button
                @click="handleDeleteLimit( limit )"
                :disabled="deleteMutation.isPending.value"
                variant="ghost"
                size="sm"
                class="text-destructive hover:text-destructive"
              >
                Delete
              </Button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else class="bg-white rounded-lg shadow p-6 text-center">
      <p class="text-gray-600 mb-4">No limits configured</p>
      <Button @click="showCreateModal = true">
        Create First Limit
      </Button>
    </div>

    <!-- Create limit modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create New Limit</DialogTitle>
          <DialogDescription>
            Set usage limits for tokens, requests, or cost. At least one limit must be specified.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="createError" variant="destructive">
          <AlertDescription>{{ createError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="project">Project ID (optional)</Label>
            <Input
              id="project"
              v-model="projectId"
              placeholder="my-project"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-2">
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

          <div class="space-y-2">
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

          <div class="space-y-2">
            <Label for="maxCostPerMonth">Max Cost per Month in cents (optional)</Label>
            <Input
              id="maxCostPerMonth"
              v-model.number="maxCostPerMonthCents"
              type="number"
              min="1"
              placeholder="e.g., 10000 for $100.00"
              :disabled="createMutation.isPending.value"
            />
            <p v-if="maxCostPerMonthCents" class="text-sm text-gray-500">
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
            Cancel
          </Button>
          <Button
            @click="handleCreateLimit"
            :disabled="createMutation.isPending.value"
          >
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

        <Alert v-if="editError" variant="destructive">
          <AlertDescription>{{ editError }}</AlertDescription>
        </Alert>

        <div v-if="editingLimit" class="space-y-4 py-4">
          <div class="space-y-2">
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

          <div class="space-y-2">
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

          <div class="space-y-2">
            <Label for="editMaxCostPerMonth">Max Cost per Month in cents (optional)</Label>
            <Input
              id="editMaxCostPerMonth"
              v-model.number="maxCostPerMonthCents"
              type="number"
              min="1"
              placeholder="e.g., 10000 for $100.00"
              :disabled="updateMutation.isPending.value"
            />
            <p v-if="maxCostPerMonthCents" class="text-sm text-gray-500">
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
            Cancel
          </Button>
          <Button
            @click="handleUpdateLimit"
            :disabled="updateMutation.isPending.value"
          >
            {{ updateMutation.isPending.value ? 'Updating...' : 'Update Limit' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
