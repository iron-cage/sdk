<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useApi, type InvitePreviewResponse } from '@/composables/useApi'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'

const route = useRoute()
const router = useRouter()
const api = useApi()
const authStore = useAuthStore()

const token = computed(() => String(route.params.token ?? ''))

const previewLoading = ref(true)
const previewError = ref<string | null>(null)
const preview = ref<InvitePreviewResponse | null>(null)

const password = ref('')
const passwordConfirm = ref('')
const submitting = ref(false)
const submitError = ref<string | null>(null)

const passwordsMatch = computed(
  () => password.value.length >= 8 && password.value === passwordConfirm.value,
)

const canSubmit = computed(() => !submitting.value && !!preview.value && passwordsMatch.value)

onMounted(async () => {
  if (!token.value) {
    previewError.value = 'Missing invite token'
    previewLoading.value = false
    return
  }
  try {
    preview.value = await api.getInvitePreview(token.value)
  } catch (e) {
    previewError.value = e instanceof Error ? e.message : String(e)
  } finally {
    previewLoading.value = false
  }
})

async function onAccept() {
  if (!canSubmit.value) return
  submitting.value = true
  submitError.value = null
  try {
    const result = await api.acceptInvite(token.value, { password: password.value })
    authStore.setMemberSession(result.access_token, result.user_id, 'developer')
    router.push('/dashboard')
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
        <CardTitle>Join Workspace</CardTitle>
        <CardDescription v-if="preview">
          You have been invited to <span class="font-medium">{{ preview.workspace_name }}</span>.
        </CardDescription>
        <CardDescription v-else>Validating invite link...</CardDescription>
      </CardHeader>

      <CardContent class="space-y-5">
        <div v-if="previewLoading" class="text-sm text-muted-foreground">
          Loading invite details...
        </div>

        <Alert v-else-if="previewError" variant="destructive">
          <AlertDescription>{{ previewError }}</AlertDescription>
        </Alert>

        <template v-else-if="preview">
          <div class="flex flex-wrap gap-2 text-xs">
            <Badge variant="secondary">{{ preview.domain }}</Badge>
            <Badge v-if="preview.default_model" variant="secondary">
              Default: {{ preview.default_model }}
            </Badge>
            <Badge variant="secondary">{{ preview.seats_remaining }} seats left</Badge>
          </div>

          <div class="space-y-2">
            <Label for="invite-password">Password</Label>
            <Input id="invite-password" v-model="password" type="password" autocomplete="new-password"
              :disabled="submitting" />
            <p class="text-xs text-muted-foreground">Minimum 8 characters.</p>
          </div>

          <div class="space-y-2">
            <Label for="invite-password-confirm">Confirm Password</Label>
            <Input id="invite-password-confirm" v-model="passwordConfirm" type="password" autocomplete="new-password"
              :disabled="submitting" />
            <p v-if="passwordConfirm && password !== passwordConfirm" class="text-xs text-destructive">
              Passwords do not match.
            </p>
          </div>

          <Alert v-if="submitError" variant="destructive">
            <AlertDescription>{{ submitError }}</AlertDescription>
          </Alert>

          <div class="flex justify-end pt-2">
            <Button :disabled="!canSubmit" @click="onAccept">
              {{ submitting ? 'Joining...' : 'Join Workspace' }}
            </Button>
          </div>
        </template>
      </CardContent>
    </Card>
  </div>
</template>
