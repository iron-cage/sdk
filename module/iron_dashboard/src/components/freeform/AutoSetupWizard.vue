<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Check, Copy } from 'lucide-vue-next'
import {
  useApi,
  type WorkspaceFreeformResponse,
  type ApplyProvidersResponse,
  type WorkspaceBudgetResponse,
  type GenerateInviteResponse,
} from '@/composables/useApi'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import FreeFormDialog from './FreeFormDialog.vue'

type Step = 'company' | 'providers' | 'budget' | 'invite' | 'done'

interface Results {
  company: WorkspaceFreeformResponse | null
  providers: ApplyProvidersResponse | null
  budget: WorkspaceBudgetResponse | null
  invite: GenerateInviteResponse | null
}

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
  (e: 'complete', results: Results): void
}>()

const api = useApi()

const step = ref<Step>('company')
const parsing = ref(false)
const error = ref<string | null>(null)
const inviteSeats = ref(10)
const inviteExpiryHours = ref<number | undefined>(168)
const copied = ref(false)

const results = ref<Results>({
  company: null,
  providers: null,
  budget: null,
  invite: null,
})

watch(
  () => props.open,
  (isOpen: boolean) => {
    if (isOpen) {
      step.value = 'company'
      parsing.value = false
      error.value = null
      inviteSeats.value = 10
      inviteExpiryHours.value = 168
      copied.value = false
      results.value = { company: null, providers: null, budget: null, invite: null }
    }
  }
)

const companyOpen = computed(() => props.open && step.value === 'company')
const providersOpen = computed(() => props.open && step.value === 'providers')
const budgetOpen = computed(() => props.open && step.value === 'budget')
const inviteOpen = computed(() => props.open && (step.value === 'invite' || step.value === 'done'))

function cancelAll() {
  emit('update:open', false)
}

async function onCompanyParse(text: string) {
  parsing.value = true
  error.value = null
  try {
    results.value.company = await api.freeformCompany(text)
    step.value = 'providers'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    parsing.value = false
  }
}

async function onProvidersParse(text: string) {
  parsing.value = true
  error.value = null
  try {
    results.value.providers = await api.freeformProviders(text)
    step.value = 'budget'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    parsing.value = false
  }
}

async function onBudgetParse(text: string) {
  parsing.value = true
  error.value = null
  try {
    results.value.budget = await api.setWorkspaceBudget({ text })
    step.value = 'invite'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    parsing.value = false
  }
}

async function generateInvite() {
  parsing.value = true
  error.value = null
  try {
    results.value.invite = await api.generateInvite({
      seats: inviteSeats.value,
      expires_in_hours: inviteExpiryHours.value,
    })
    step.value = 'done'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    parsing.value = false
  }
}

async function copyInviteUrl() {
  const invite = results.value.invite
  if (!invite) return
  try {
    await navigator.clipboard.writeText(invite.url)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 1500)
  } catch {
    error.value = 'Failed to copy to clipboard'
  }
}

function finish() {
  emit('complete', results.value)
  emit('update:open', false)
}
</script>

<template>
  <!-- Step 1: company -->
  <FreeFormDialog
    :open="companyOpen"
    title="Step 1 of 4 - Company"
    description="Paste company name, domain, and account type. Example: Acme Corp, acme.com, Client"
    placeholder="Acme Corp, acme.com, Client"
    parse-label="Parse and Continue"
    :parsing="parsing"
    :error="error"
    @parse="onCompanyParse"
    @cancel="cancelAll"
  />

  <!-- Step 2: providers -->
  <FreeFormDialog
    :open="providersOpen"
    title="Step 2 of 4 - Providers"
    description="Paste API keys for each provider, one per line."
    :placeholder="`openai: sk-proj-...\nanthropic: sk-ant-...\ngemini: AIza...`"
    parse-label="Parse and Continue"
    :parsing="parsing"
    :error="error"
    @parse="onProvidersParse"
    @cancel="cancelAll"
  />

  <!-- Step 3: budget -->
  <FreeFormDialog
    :open="budgetOpen"
    title="Step 3 of 4 - Budget"
    description="Set the workspace spending limit. Example: limit all users $100/week"
    placeholder="limit all users $100/week"
    parse-label="Parse and Continue"
    :parsing="parsing"
    :error="error"
    @parse="onBudgetParse"
    @cancel="cancelAll"
  />

  <!-- Step 4: invite (form + result) -->
  <Dialog
    :open="inviteOpen"
    @update:open="(v: boolean) => v || cancelAll()"
  >
    <DialogContent class="sm:max-w-xl">
      <DialogHeader>
        <DialogTitle>Step 4 of 4 - Invite Link</DialogTitle>
        <DialogDescription>
          Generate a magic invite link that members can use to join your workspace.
        </DialogDescription>
      </DialogHeader>

      <div
        v-if="step === 'invite'"
        class="space-y-4"
      >
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-2">
            <label
              class="text-sm font-medium"
              for="auto-setup-seats"
              >Seats</label
            >
            <input
              id="auto-setup-seats"
              v-model.number="inviteSeats"
              type="number"
              min="1"
              max="1000"
              class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm"
            />
          </div>
          <div class="space-y-2">
            <label
              class="text-sm font-medium"
              for="auto-setup-expiry"
              >Expires in (hours)</label
            >
            <input
              id="auto-setup-expiry"
              v-model.number="inviteExpiryHours"
              type="number"
              min="1"
              class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm"
            />
          </div>
        </div>
        <p
          v-if="error"
          class="text-sm text-destructive"
        >
          {{ error }}
        </p>
      </div>

      <div
        v-else-if="step === 'done' && results.invite"
        class="space-y-4"
      >
        <div class="rounded-md border bg-muted/30 p-3 font-mono text-sm break-all">
          {{ results.invite.url }}
        </div>
        <div class="flex flex-wrap gap-2 text-xs">
          <Badge variant="secondary">{{ results.invite.seats_total }} seats</Badge>
          <Badge
            v-if="results.invite.expires_at"
            variant="secondary"
          >
            Expires {{ new Date(results.invite.expires_at).toLocaleString() }}
          </Badge>
          <Badge
            v-else
            variant="secondary"
            >No expiry</Badge
          >
        </div>
      </div>

      <DialogFooter>
        <template v-if="step === 'invite'">
          <Button
            type="button"
            variant="ghost"
            :disabled="parsing"
            @click="cancelAll"
          >
            Cancel
          </Button>
          <Button
            type="button"
            :disabled="parsing || inviteSeats < 1"
            @click="generateInvite"
          >
            {{ parsing ? 'Generating...' : 'Generate' }}
          </Button>
        </template>
        <template v-else>
          <Button
            type="button"
            variant="outline"
            @click="copyInviteUrl"
          >
            <component
              :is="copied ? Check : Copy"
              class="size-4"
            />
            {{ copied ? 'Copied' : 'Copy Link' }}
          </Button>
          <Button
            type="button"
            @click="finish"
            >Done</Button
          >
        </template>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
