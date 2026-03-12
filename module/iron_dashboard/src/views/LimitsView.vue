<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type BudgetStatus } from '../composables/useApi'
import { useAuthStore } from '../stores/auth'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
const budgetError = ref('')

// Fetch agent budget status
const { data: budgetStatus, isLoading: isBudgetLoading, error: budgetQueryError, refetch: refetchBudget } = useQuery({
  queryKey: ['budget-status'],
  queryFn: () => api.getBudgetStatus(),
})

function openBudgetModal(row: BudgetStatus) {
  budgetAgentId.value = row.agent_id
  budgetAgentName.value = row.agent_name
  budgetUsd.value = Number((row.budget / 1_000_000).toFixed(2))
  budgetError.value = ''
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
  budgetError.value = ''
  if (!budgetUsd.value || budgetUsd.value <= 0) {
    budgetError.value = 'Budget must be greater than zero'
    return
  }

  const micros = Math.round(budgetUsd.value * 1_000_000)
  updateBudgetMutation.mutate({
    agentId: budgetAgentId.value,
    total_allocated_microdollars: micros,
  })
}

function riskBadgeVariant(risk: string) {
  switch (risk) {
    case 'exhausted':
    case 'critical': return 'bg-destructive'
    case 'high':
    case 'medium':   return 'bg-warning'
    case 'low':      return 'bg-success'
    default:         return 'bg-muted'
  }
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

    <!-- Budget summary bar -->
    <div v-if="budgetStatus?.summary" class="flex flex-wrap items-center gap-2 px-4 py-3 border-b border-border bg-muted/30">
      <span class="text-sm font-medium text-foreground mr-1">
        {{ budgetStatus.summary.total_agents }} agents
      </span>
      <div class="h-4 w-px bg-border mx-1" />
      <span
        v-if="budgetStatus.summary.exhausted"
        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-destructive  text-xs font-medium"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-destructive" />
        {{ budgetStatus.summary.exhausted }} exhausted
      </span>
      <span
        v-if="budgetStatus.summary.critical"
        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-destructive text-xs font-medium"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-destructive" />
        {{ budgetStatus.summary.critical }} critical
      </span>
      <span
        v-if="budgetStatus.summary.high"
        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-warning text-xs font-medium"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-warning" />
        {{ budgetStatus.summary.high }} high
      </span>
      <span
        v-if="budgetStatus.summary.medium"
        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-warning text-xs font-medium"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-warning/60" />
        {{ budgetStatus.summary.medium }} medium
      </span>
      <span
        v-if="budgetStatus.summary.low"
        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-success text-xs font-medium"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-success" />
        {{ budgetStatus.summary.low }} low
      </span>
      <span
        v-if="budgetStatus.summary.active !== undefined"
        class="ml-auto text-xs text-muted-foreground"
      >
        {{ budgetStatus.summary.active }} active · {{ budgetStatus.summary.exhausted }} exhausted
      </span>
    </div>

    <!-- Agent Budgets -->
    <DataTable
      :columns="[
        { label: 'Agent' },
        { label: 'Status' },
        { label: 'Allocated' },
        { label: 'Spent' },
        { label: 'Remaining' },
        { label: 'Used' },
        { label: 'Risk' },
        { label: 'Actions', align: 'right' },
      ]"
      :is-loading="isBudgetLoading"
      :error="budgetQueryError"
      :is-empty="!budgetStatus?.data?.length"
      loading-text="Loading agent budgets..."
    >
      <template #empty>
        <p class="text-muted-foreground">No agent budget data available.</p>
      </template>
      <tr v-for="row in budgetStatus?.data" :key="row.agent_id">
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base font-medium text-foreground">
          {{ row.agent_name }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap">
          <Badge variant="outline" class="capitalize">{{ row.status }}</Badge>
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
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap ">
          <span class="capitalize text-sm flex gap-2 items-center">
            <span :class="`rounded-full h-2 w-2 inline-block ${riskBadgeVariant(row.risk_level)}`"></span>
            {{ row.risk_level }}
          </span>
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

        <p v-if="budgetError" class="text-sm text-destructive">{{ budgetError }}</p>

        <DialogFooter>
          <Button variant="outline" @click="showBudgetModal = false; budgetError = ''">
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
