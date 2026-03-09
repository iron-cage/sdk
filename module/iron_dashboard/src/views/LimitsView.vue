<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type BudgetStatus } from '../composables/useApi'
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
import IconX from '@/components/icons/IconX.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconDotsHorizontal from '@/components/icons/IconDotsHorizontal.vue'
import IconEdit from '@/components/icons/IconEdit.vue'
import IconRefresh from '@/components/icons/IconRefresh.vue'
import PageLayout from '@/components/PageLayout.vue'
import DataTable from '@/components/DataTable.vue'
import PercentBar from '@/components/PercentBar.vue'

const api = useApi()
const authStore = useAuthStore()
const queryClient = useQueryClient()

const showBudgetModal = ref(false)
const budgetAgentId = ref<number | null>(null)
const budgetAgentName = ref('')
const budgetUsd = ref<number | undefined>(undefined)

// Fetch agent budget status
const { data: budgetStatus, isLoading: isBudgetLoading, error: budgetQueryError, refetch: refetchBudget } = useQuery({
  queryKey: ['budget-status'],
  queryFn: () => api.getBudgetStatus(),
})

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
        <IconRefresh />
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
                <IconDotsHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem @click="openBudgetModal(row)">
                <IconEdit />
                Update Budget
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </td>
      </tr>
    </DataTable>

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
            <IconX />
            Cancel
          </Button>
          <Button @click="handleUpdateBudget">
            <IconCheck />
            Update Budget
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </PageLayout>
</template>
