<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useApi, type BudgetPeriod } from '@/composables/useApi'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Alert, AlertDescription } from '@/components/ui/alert'

const api = useApi()
const router = useRouter()

const amountDollars = ref<number | undefined>(100)
const period = ref<BudgetPeriod>('week')

const submitting = ref(false)
const submitError = ref<string | null>(null)
const submitted = ref(false)

const canSubmit = computed(
  () =>
    !submitting.value &&
    amountDollars.value !== undefined &&
    !Number.isNaN(amountDollars.value) &&
    amountDollars.value > 0
)

async function onSubmit() {
  if (!canSubmit.value || amountDollars.value === undefined) return
  submitting.value = true
  submitError.value = null
  try {
    await api.setWorkspaceBudget({
      amount_cents: Math.round(amountDollars.value * 100),
      period: period.value,
    })
    submitted.value = true
    setTimeout(() => {
      router.push('/dashboard')
    }, 600)
  } catch (e) {
    submitError.value = e instanceof Error ? e.message : String(e)
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="mx-auto max-w-xl py-10">
    <Card>
      <CardHeader>
        <CardTitle>Workspace Budget</CardTitle>
        <CardDescription>
          Set a spending limit that applies to every member of the workspace.
        </CardDescription>
      </CardHeader>

      <CardContent class="space-y-5">
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-2">
            <Label for="budget-amount">Amount (USD)</Label>
            <div class="relative">
              <span
                class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-sm text-muted-foreground"
              >
                $
              </span>
              <Input
                id="budget-amount"
                v-model.number="amountDollars"
                type="number"
                min="1"
                step="1"
                placeholder="100"
                class="pl-7"
                :disabled="submitting"
              />
            </div>
          </div>

          <div class="space-y-2">
            <Label for="budget-period">Period</Label>
            <Select
              v-model="period"
              :disabled="submitting"
            >
              <SelectTrigger id="budget-period">
                <SelectValue placeholder="Select period" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="day">Per day</SelectItem>
                <SelectItem value="week">Per week</SelectItem>
                <SelectItem value="month">Per month</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <Alert
          v-if="submitError"
          variant="destructive"
        >
          <AlertDescription>{{ submitError }}</AlertDescription>
        </Alert>

        <Alert v-if="submitted">
          <AlertDescription>Budget saved. Redirecting...</AlertDescription>
        </Alert>

        <div class="flex justify-end pt-2">
          <Button
            :disabled="!canSubmit"
            @click="onSubmit"
          >
            {{ submitting ? 'Saving...' : 'Save Budget' }}
          </Button>
        </div>
      </CardContent>
    </Card>
  </div>
</template>
