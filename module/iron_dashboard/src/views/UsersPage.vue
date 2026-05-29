<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Check, Copy, Plus } from 'lucide-vue-next'
import { useApi, type GenerateInviteResponse, type PendingInviteSeat } from '@/composables/useApi'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
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

const dialogOpen = ref(false)
const seats = ref<number | undefined>(10)
const expiryHours = ref<number | undefined>(168)

const submitting = ref(false)
const submitError = ref<string | null>(null)
const invite = ref<GenerateInviteResponse | null>(null)
const copied = ref(false)

const absoluteUrl = computed(() => {
  if (!invite.value) return ''
  try {
    return new URL(invite.value.url, window.location.origin).toString()
  } catch {
    return invite.value.url
  }
})

const canSubmit = computed(
  () =>
    !submitting.value && seats.value !== undefined && !Number.isNaN(seats.value) && seats.value >= 1
)

function openDialog() {
  invite.value = null
  submitError.value = null
  copied.value = false
  seats.value = 10
  expiryHours.value = 168
  dialogOpen.value = true
}

async function onGenerate() {
  if (!canSubmit.value || seats.value === undefined) return
  submitting.value = true
  submitError.value = null
  try {
    invite.value = await api.generateInvite({
      seats: seats.value,
      expires_in_hours: expiryHours.value,
    })
  } catch (e) {
    submitError.value = e instanceof Error ? e.message : String(e)
  } finally {
    submitting.value = false
  }
}

async function copyLink() {
  if (!absoluteUrl.value) return
  try {
    await navigator.clipboard.writeText(absoluteUrl.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 1500)
  } catch {
    submitError.value = 'Failed to copy to clipboard'
  }
}

// Pending members section
const pendingLoading = ref(true)
const pendingError = ref<string | null>(null)
const pending = ref<PendingInviteSeat[]>([])
const approvingSeatId = ref<number | null>(null)

async function refreshPending() {
  pendingLoading.value = true
  pendingError.value = null
  try {
    const result = await api.listPendingInvites()
    pending.value = result.pending
  } catch (e) {
    pendingError.value = e instanceof Error ? e.message : String(e)
  } finally {
    pendingLoading.value = false
  }
}

async function approveSeat(seat: PendingInviteSeat) {
  approvingSeatId.value = seat.seat_id
  pendingError.value = null
  try {
    await api.approveInviteSeat(seat.seat_id)
    pending.value = pending.value.filter((p) => p.seat_id !== seat.seat_id)
  } catch (e) {
    pendingError.value = e instanceof Error ? e.message : String(e)
  } finally {
    approvingSeatId.value = null
  }
}

onMounted(() => {
  refreshPending()
})
</script>

<template>
  <div class="mx-auto max-w-2xl py-10">
    <Card>
      <CardHeader>
        <div class="flex items-start justify-between gap-4">
          <div class="space-y-1">
            <CardTitle>Users</CardTitle>
            <CardDescription>
              Invite team members to your workspace via magic link.
            </CardDescription>
          </div>
          <Button @click="openDialog">
            <Plus class="size-4" />
            Invite
          </Button>
        </div>
      </CardHeader>

      <CardContent>
        <div class="rounded-md border border-dashed bg-muted/30 p-8 text-center">
          <p class="text-sm text-muted-foreground">
            Click <span class="font-medium">Invite</span> to generate a magic link team members can
            use to join.
          </p>
        </div>
      </CardContent>
    </Card>

    <Card class="mt-6">
      <CardHeader>
        <div class="flex items-center justify-between gap-4">
          <div class="space-y-1">
            <CardTitle>Pending Approval</CardTitle>
            <CardDescription>
              Members who joined via invite link and are waiting for an admin.
            </CardDescription>
          </div>
          <Button
            variant="ghost"
            size="sm"
            :disabled="pendingLoading"
            @click="refreshPending"
          >
            {{ pendingLoading ? 'Refreshing...' : 'Refresh' }}
          </Button>
        </div>
      </CardHeader>

      <CardContent class="space-y-3">
        <Alert
          v-if="pendingError"
          variant="destructive"
        >
          <AlertDescription>{{ pendingError }}</AlertDescription>
        </Alert>

        <div
          v-if="pendingLoading && pending.length === 0"
          class="text-sm text-muted-foreground"
        >
          Loading pending members...
        </div>

        <div
          v-else-if="!pendingLoading && pending.length === 0"
          class="rounded-md border border-dashed bg-muted/30 p-6 text-center text-sm text-muted-foreground"
        >
          No pending members.
        </div>

        <ul
          v-else
          class="divide-y rounded-md border"
        >
          <li
            v-for="seat in pending"
            :key="seat.seat_id"
            class="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between"
          >
            <div class="space-y-1">
              <div class="text-sm font-medium">{{ seat.username || seat.user_id }}</div>
              <div class="text-xs text-muted-foreground">{{ seat.email }}</div>
              <div class="text-xs text-muted-foreground">
                {{ seat.workspace_name }}
                <span v-if="seat.accepted_at">
                  - accepted {{ new Date(seat.accepted_at).toLocaleString() }}
                </span>
              </div>
            </div>
            <Button
              size="sm"
              :disabled="approvingSeatId === seat.seat_id"
              @click="approveSeat(seat)"
            >
              {{ approvingSeatId === seat.seat_id ? 'Approving...' : 'Approve' }}
            </Button>
          </li>
        </ul>
      </CardContent>
    </Card>

    <Dialog v-model:open="dialogOpen">
      <DialogContent class="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>Generate Invite Link</DialogTitle>
          <DialogDescription>
            Set how many people can use the link and when it expires.
          </DialogDescription>
        </DialogHeader>

        <div
          v-if="!invite"
          class="space-y-4"
        >
          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <Label for="invite-seats">Seats</Label>
              <Input
                id="invite-seats"
                v-model.number="seats"
                type="number"
                min="1"
                max="1000"
                :disabled="submitting"
              />
            </div>
            <div class="space-y-2">
              <Label for="invite-expiry">Expires in (hours)</Label>
              <Input
                id="invite-expiry"
                v-model.number="expiryHours"
                type="number"
                min="1"
                :disabled="submitting"
              />
            </div>
          </div>

          <Alert
            v-if="submitError"
            variant="destructive"
          >
            <AlertDescription>{{ submitError }}</AlertDescription>
          </Alert>
        </div>

        <div
          v-else
          class="space-y-4"
        >
          <div class="rounded-md border bg-muted/30 p-3 font-mono text-sm break-all">
            {{ absoluteUrl }}
          </div>
          <div class="flex flex-wrap gap-2 text-xs">
            <Badge variant="secondary">{{ invite.seats_total }} seats</Badge>
            <Badge
              v-if="invite.expires_at"
              variant="secondary"
            >
              Expires {{ new Date(invite.expires_at).toLocaleString() }}
            </Badge>
            <Badge
              v-else
              variant="secondary"
              >No expiry</Badge
            >
          </div>
          <Alert
            v-if="submitError"
            variant="destructive"
          >
            <AlertDescription>{{ submitError }}</AlertDescription>
          </Alert>
        </div>

        <DialogFooter>
          <template v-if="!invite">
            <Button
              variant="ghost"
              :disabled="submitting"
              @click="dialogOpen = false"
            >
              Cancel
            </Button>
            <Button
              :disabled="!canSubmit"
              @click="onGenerate"
            >
              {{ submitting ? 'Generating...' : 'Generate' }}
            </Button>
          </template>
          <template v-else>
            <Button
              variant="outline"
              @click="copyLink"
            >
              <component
                :is="copied ? Check : Copy"
                class="size-4"
              />
              {{ copied ? 'Copied' : 'Copy Link' }}
            </Button>
            <Button @click="dialogOpen = false">Done</Button>
          </template>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
