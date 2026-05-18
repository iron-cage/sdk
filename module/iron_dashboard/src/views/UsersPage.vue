<script setup lang="ts">
import { ref, computed } from 'vue'
import { Check, Copy, Plus } from 'lucide-vue-next'
import { useApi, type GenerateInviteResponse } from '@/composables/useApi'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
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
    !submitting.value &&
    seats.value !== undefined &&
    !Number.isNaN(seats.value) &&
    seats.value >= 1,
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
            Click <span class="font-medium">Invite</span> to generate a magic link team
            members can use to join.
          </p>
        </div>
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

        <div v-if="!invite" class="space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <Label for="invite-seats">Seats</Label>
              <Input id="invite-seats" v-model.number="seats" type="number" min="1" max="1000" :disabled="submitting" />
            </div>
            <div class="space-y-2">
              <Label for="invite-expiry">Expires in (hours)</Label>
              <Input id="invite-expiry" v-model.number="expiryHours" type="number" min="1" :disabled="submitting" />
            </div>
          </div>

          <Alert v-if="submitError" variant="destructive">
            <AlertDescription>{{ submitError }}</AlertDescription>
          </Alert>
        </div>

        <div v-else class="space-y-4">
          <div class="rounded-md border bg-muted/30 p-3 font-mono text-sm break-all">
            {{ absoluteUrl }}
          </div>
          <div class="flex flex-wrap gap-2 text-xs">
            <Badge variant="secondary">{{ invite.seats_total }} seats</Badge>
            <Badge v-if="invite.expires_at" variant="secondary">
              Expires {{ new Date(invite.expires_at).toLocaleString() }}
            </Badge>
            <Badge v-else variant="secondary">No expiry</Badge>
          </div>
          <Alert v-if="submitError" variant="destructive">
            <AlertDescription>{{ submitError }}</AlertDescription>
          </Alert>
        </div>

        <DialogFooter>
          <template v-if="!invite">
            <Button variant="ghost" :disabled="submitting" @click="dialogOpen = false">
              Cancel
            </Button>
            <Button :disabled="!canSubmit" @click="onGenerate">
              {{ submitting ? 'Generating...' : 'Generate' }}
            </Button>
          </template>
          <template v-else>
            <Button variant="outline" @click="copyLink">
              <component :is="copied ? Check : Copy" class="size-4" />
              {{ copied ? 'Copied' : 'Copy Link' }}
            </Button>
            <Button @click="dialogOpen = false">Done</Button>
          </template>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
