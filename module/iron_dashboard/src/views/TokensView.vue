<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type TokenMetadata, type CreateTokenResponse } from '../composables/useApi'
import { useAuthStore } from '../stores/auth'
import PageLayout from '@/components/PageLayout.vue'
import DataTable from '@/components/DataTable.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from 'vue-sonner'
import { formatDate } from '@/lib/formatters'
import { useConfirm } from '@/composables/useConfirm'
import IconPlus from '@/components/icons/IconPlus.vue'
import IconX from '@/components/icons/IconX.vue'
import IconKey from '@/components/icons/IconKey.vue'
import IconDotsHorizontal from '@/components/icons/IconDotsHorizontal.vue'
import IconRefresh from '@/components/icons/IconRefresh.vue'
import IconBan from '@/components/icons/IconBan.vue'
import IconCopy from '@/components/icons/IconCopy.vue'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

const api = useApi()
const authStore = useAuthStore()
const queryClient = useQueryClient()

const showCreateModal = ref(false)
const showTokenModal = ref(false)
const { showConfirmModal, confirmTitle, confirmDescription, confirmLabel, confirmVariant, confirmCallback, openConfirm } = useConfirm()
const newTokenData = ref<CreateTokenResponse | null>(null)
const projectId = ref('')
const description = ref('')
const selectedUserId = ref('')

// Fetch users for dropdown
const { data: usersList } = useQuery({
  queryKey: ['users-list'],
  queryFn: () => api.getUsers({ page: 1, page_size: 100 }),
})

// Fetch tokens
const { data: tokens, isLoading, error, refetch } = useQuery({
  queryKey: ['tokens'],
  queryFn: () => api.getTokens(),
})

// Create token mutation
const createMutation = useMutation({
  mutationFn: (data: { user_id: string; project_id?: string; description?: string }) =>
    api.createToken(data),
  onSuccess: (data) => {
    newTokenData.value = data
    showCreateModal.value = false
    showTokenModal.value = true
    projectId.value = ''
    description.value = ''
    selectedUserId.value = authStore.userId || ''
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create token')
  },
})

// Rotate token mutation
const rotateMutation = useMutation({
  mutationFn: (id: number) => api.rotateToken(id),
  onSuccess: (data) => {
    newTokenData.value = data
    showTokenModal.value = true
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  },
})

// Revoke token mutation
const revokeMutation = useMutation({
  mutationFn: (id: number) => api.revokeToken(id),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  },
})

function handleCreateToken() {
  const userId = selectedUserId.value || authStore.userId
  if (!userId) {
    toast.error('Cannot create token: no authenticated user')
    return
  }
  createMutation.mutate({
    user_id: userId,
    project_id: projectId.value || undefined,
    description: description.value || undefined,
  })
}

function handleRotateToken(token: TokenMetadata) {
  openConfirm(
    'Rotate Token',
    `Rotate token ${token.id}? The current token will be revoked and a new one issued.`,
    'Rotate',
    () => rotateMutation.mutate(token.id),
  )
}

function handleRevokeToken(token: TokenMetadata) {
  openConfirm(
    'Revoke Token',
    `Revoke token ${token.id}? This action cannot be undone.`,
    'Revoke',
    () => revokeMutation.mutate(token.id),
  )
}

function copyToken(token: string) {
  navigator.clipboard.writeText(token)
}
</script>

<template>
  <PageLayout title="Token Management">
    <template #actions>
      <Button @click="showCreateModal = true">
        <IconPlus />
        Generate New Token
      </Button>
    </template>

    <DataTable
      :columns="[
        { label: 'ID' },
        { label: 'Provider' },
        { label: 'Description' },
        { label: 'Created' },
        { label: 'Status' },
        { label: 'Actions', align: 'right' },
      ]"
      :is-loading="isLoading"
      :error="error"
      :is-empty="!tokens || tokens.length === 0"
      loading-text="Loading tokens..."
      :on-retry="() => refetch()"
    >
      <template #empty>
        <p class="text-muted-foreground mb-4">No tokens found</p>
        <Button @click="showCreateModal = true"><IconPlus />Generate First Token</Button>
      </template>

      <tr v-for="token in tokens" :key="token.id">
        <td class="px-6 py-4 whitespace-nowrap text-base text-foreground">{{ token.id }}</td>
        <td class="px-6 py-4 whitespace-nowrap text-base text-foreground">
          <Badge variant="outline">{{ token.provider || '-' }}</Badge>
        </td>
        <td class="px-6 py-4 whitespace-nowrap text-base text-foreground">{{ token.name || '-' }}</td>
        <td class="px-6 py-4 whitespace-nowrap text-base text-muted-foreground">{{ formatDate(token.created_at) }}</td>
        <td class="px-6 py-4 whitespace-nowrap">
          <Badge :variant="token.is_active ? 'default' : 'destructive'">
            {{ token.is_active ? 'Active' : 'Revoked' }}
          </Badge>
        </td>
        <td class="px-6 py-4 whitespace-nowrap text-right text-base font-medium">
          <DropdownMenu v-if="token.is_active">
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm">
                <span class="sr-only">Open menu</span>
                <IconDotsHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuLabel>Actions</DropdownMenuLabel>
              <DropdownMenuItem :disabled="rotateMutation.isPending.value" @click="handleRotateToken(token)">
                <IconRefresh />
                Rotate
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem :disabled="revokeMutation.isPending.value" class="text-destructive" @click="handleRevokeToken(token)">
                <IconBan />
                Revoke
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </td>
      </tr>
    </DataTable>

    <!-- Create token modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Generate New Token</DialogTitle>
          <DialogDescription>
            Create a new API token with optional project ID and description.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4">
          <div class="space-y-1.5">
            <Label for="user">User</Label>
            <Select v-model="selectedUserId">
              <SelectTrigger id="user">
                <SelectValue placeholder="Select a user" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem 
                  v-for="user in usersList?.users" 
                  :key="user.id" 
                  :value="user.username"
                >
                  {{ user.username }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-1.5">
            <Label for="project">Project ID (optional)</Label>
            <Input
              id="project"
              v-model="projectId"
              placeholder="my-project"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="description">Description (optional)</Label>
            <Input
              id="description"
              v-model="description"
              placeholder="Production API token"
              :disabled="createMutation.isPending.value"
            />
          </div>
        </div>

        <DialogFooter>
          <Button
            :disabled="createMutation.isPending.value"
            variant="outline"
            @click="showCreateModal = false"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="createMutation.isPending.value"
            @click="handleCreateToken"
          >
            <IconKey />
            {{ createMutation.isPending.value ? 'Generating...' : 'Generate Token' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- New token modal -->
    <Dialog v-model:open="showTokenModal">
      <DialogContent class="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Token Generated</DialogTitle>
          <DialogDescription>
            Save this token now. You won't be able to see it again!
          </DialogDescription>
        </DialogHeader>

        <div class="rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-foreground">
          <strong>Important:</strong> Copy this token now — it won't be shown again.
        </div>

        <div v-if="newTokenData" class="space-y-4">
          <div class="space-y-1.5">
            <Label>Token</Label>
            <div class="flex space-x-2">
              <Input
                v-model="newTokenData.token"
                readonly
                class="font-mono text-base bg-muted"
              />
              <Button
                variant="outline"
                @click="copyToken(newTokenData.token)"
              >
                <IconCopy />
                Copy
              </Button>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4 text-base">
            <div>
              <span class="text-muted-foreground">ID:</span>
              <span class="ml-2 font-medium">{{ newTokenData.id }}</span>
            </div>
            <div>
              <span class="text-muted-foreground">User:</span>
              <span class="ml-2 font-medium">{{ newTokenData.user_id }}</span>
            </div>
            <div v-if="newTokenData.provider">
              <span class="text-muted-foreground">Provider:</span>
              <span class="ml-2 font-medium">{{ newTokenData.provider }}</span>
            </div>
            <div v-if="newTokenData.description">
              <span class="text-muted-foreground">Description:</span>
              <span class="ml-2 font-medium">{{ newTokenData.description }}</span>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button @click="showTokenModal = false; newTokenData = null">
            <IconX />
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <ConfirmDialog
      v-model:open="showConfirmModal"
      :title="confirmTitle"
      :description="confirmDescription"
      :confirm-label="confirmLabel"
      :variant="confirmVariant"
      @confirm="confirmCallback?.()"
    />
  </PageLayout>
</template>
