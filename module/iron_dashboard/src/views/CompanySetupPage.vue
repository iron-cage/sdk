<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useApi } from '@/composables/useApi'
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
import { FreeFormDialog, FreeFormToggle } from '@/components/freeform'

type AccountType = 'client' | 'internal'

const api = useApi()
const router = useRouter()

const companyName = ref('')
const domain = ref('')
const accountType = ref<AccountType>('client')

const aiInferred = ref(false)
const freeFormOpen = ref(false)
const freeFormError = ref<string | null>(null)
const freeFormParsing = ref(false)

const submitting = ref(false)
const submitError = ref<string | null>(null)
const submitted = ref(false)

const subtitle = computed(() =>
  aiInferred.value ? 'AI inferred - review and confirm' : 'Tell us about your company.'
)

const canSubmit = computed(
  () => !submitting.value && companyName.value.trim() && domain.value.trim()
)

function openFreeForm() {
  freeFormError.value = null
  freeFormOpen.value = true
}

function buildFreeFormSeed(): string {
  if (!companyName.value && !domain.value) return ''
  return `${companyName.value}, ${domain.value}, ${accountType.value}`
}

async function onFreeFormParse(text: string) {
  freeFormParsing.value = true
  freeFormError.value = null
  try {
    const result = await api.freeformCompany(text)
    companyName.value = result.name
    domain.value = result.domain
    accountType.value = result.account_type === 'internal' ? 'internal' : 'client'
    aiInferred.value = true
    freeFormOpen.value = false
  } catch (e) {
    freeFormError.value = e instanceof Error ? e.message : String(e)
  } finally {
    freeFormParsing.value = false
  }
}

async function onConfirm() {
  submitting.value = true
  submitError.value = null
  try {
    const csv = `${companyName.value.trim()}, ${domain.value.trim()}, ${accountType.value}`
    await api.freeformCompany(csv)
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
        <div class="flex items-start justify-between gap-4">
          <div class="space-y-1">
            <CardTitle>Team Account Setup</CardTitle>
            <CardDescription>{{ subtitle }}</CardDescription>
          </div>
          <FreeFormToggle
            :active="aiInferred"
            @click="openFreeForm"
          />
        </div>
      </CardHeader>

      <CardContent class="space-y-5">
        <div class="space-y-2">
          <Label for="company-name">Company Name</Label>
          <Input
            id="company-name"
            v-model="companyName"
            placeholder="Acme Corp"
            :disabled="submitting"
          />
        </div>

        <div class="space-y-2">
          <Label for="company-domain">Company Domain</Label>
          <Input
            id="company-domain"
            v-model="domain"
            placeholder="acme.com"
            :disabled="submitting"
          />
        </div>

        <div class="space-y-2">
          <Label for="company-account-type">Account Type</Label>
          <Select
            v-model="accountType"
            :disabled="submitting"
          >
            <SelectTrigger id="company-account-type">
              <SelectValue placeholder="Select account type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="client">Client Account</SelectItem>
              <SelectItem value="internal">Internal Account</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <Alert
          v-if="submitError"
          variant="destructive"
        >
          <AlertDescription>{{ submitError }}</AlertDescription>
        </Alert>

        <Alert v-if="submitted">
          <AlertDescription>Workspace saved. Redirecting...</AlertDescription>
        </Alert>

        <div class="flex justify-end pt-2">
          <Button
            :disabled="!canSubmit"
            @click="onConfirm"
          >
            {{ submitting ? 'Saving...' : aiInferred ? 'Confirm' : 'Continue' }}
          </Button>
        </div>
      </CardContent>
    </Card>

    <FreeFormDialog
      v-model:open="freeFormOpen"
      title="Team Setup - FreeForm Mode"
      description="Paste company name, website, size, or any description."
      placeholder="Acme Corp, acme.com, Client"
      :initial-text="buildFreeFormSeed()"
      parse-label="Parse"
      :parsing="freeFormParsing"
      :error="freeFormError"
      @parse="onFreeFormParse"
    />
  </div>
</template>
