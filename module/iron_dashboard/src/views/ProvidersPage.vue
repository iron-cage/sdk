<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { CheckCircle2 } from 'lucide-vue-next'
import { useApi, type AppliedProvider } from '@/composables/useApi'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { FreeFormDialog, FreeFormToggle } from '@/components/freeform'

const api = useApi()
const router = useRouter()

const PROVIDER_LABELS: Record<string, string> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
  gemini: 'Gemini',
  google: 'Gemini',
  mistral: 'Mistral AI',
  cohere: 'Cohere',
}

const freeFormOpen = ref(false)
const freeFormParsing = ref(false)
const freeFormError = ref<string | null>(null)
const lastText = ref('')

const applied = ref<AppliedProvider[]>([])

const detected = computed(() => applied.value.length > 0)

const appliedLabels = computed(() =>
  applied.value.map((p) => PROVIDER_LABELS[p.provider.toLowerCase()] ?? p.provider)
)

function openFreeForm() {
  freeFormError.value = null
  freeFormOpen.value = true
}

async function onParse(text: string) {
  freeFormParsing.value = true
  freeFormError.value = null
  try {
    const result = await api.freeformProviders(text)
    applied.value = result.applied
    lastText.value = text
    freeFormOpen.value = false
  } catch (e) {
    freeFormError.value = e instanceof Error ? e.message : String(e)
  } finally {
    freeFormParsing.value = false
  }
}

function editLast() {
  freeFormError.value = null
  freeFormOpen.value = true
}

function done() {
  router.push('/dashboard')
}
</script>

<template>
  <div class="mx-auto max-w-2xl py-10">
    <Card>
      <CardHeader>
        <div class="flex items-start justify-between gap-4">
          <div class="space-y-1">
            <CardTitle>Inference Providers</CardTitle>
            <CardDescription>
              Register API keys so the workspace can route inference traffic.
            </CardDescription>
          </div>
          <FreeFormToggle
            :active="detected"
            @click="openFreeForm"
          />
        </div>
      </CardHeader>

      <CardContent class="space-y-5">
        <!-- Empty state -->
        <div
          v-if="!detected"
          class="rounded-md border border-dashed bg-muted/30 p-8 text-center"
        >
          <p class="text-sm text-muted-foreground">
            No providers configured yet. Paste provider keys to get started.
          </p>
          <Button
            class="mt-4"
            @click="openFreeForm"
            >Open FreeForm</Button
          >
        </div>

        <!-- Detected card -->
        <div
          v-else
          class="rounded-md border bg-emerald-50/40 p-4"
        >
          <div class="flex items-start gap-3">
            <CheckCircle2 class="size-5 shrink-0 text-emerald-600" />
            <div class="space-y-2">
              <div class="font-medium">
                {{ applied.length }} Inference Provider{{ applied.length === 1 ? '' : 's' }} applied
              </div>
              <div class="flex flex-wrap gap-2">
                <Badge
                  v-for="label in appliedLabels"
                  :key="label"
                  variant="secondary"
                >
                  {{ label }}
                </Badge>
              </div>
            </div>
          </div>
        </div>

        <Alert
          v-if="freeFormError && !freeFormOpen"
          variant="destructive"
        >
          <AlertDescription>{{ freeFormError }}</AlertDescription>
        </Alert>

        <div
          v-if="detected"
          class="flex justify-end gap-2 pt-2"
        >
          <Button
            variant="ghost"
            @click="editLast"
            >Edit</Button
          >
          <Button @click="done">Done</Button>
        </div>
      </CardContent>
    </Card>

    <FreeFormDialog
      v-model:open="freeFormOpen"
      title="Providers - FreeForm Mode"
      description="Paste API keys, one provider per line. Format: provider: key"
      :placeholder="`openai: sk-proj-...\nanthropic: sk-ant-...\ngemini: AIza...`"
      :initial-text="lastText"
      parse-label="Parse"
      :parsing="freeFormParsing"
      :error="freeFormError"
      @parse="onParse"
    />
  </div>
</template>
