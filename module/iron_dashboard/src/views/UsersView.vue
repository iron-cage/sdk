<script setup lang="ts">
import { ref, watch } from 'vue'
import { refDebounced } from '@vueuse/core'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type CreateUserRequest, type User } from '../composables/useApi'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useConfirm } from '@/composables/useConfirm'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
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
import { formatTimestamp } from '@/lib/formatters'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import PageLayout from '@/components/PageLayout.vue'
import { useAuthStore } from '../stores/auth'
import DataTable from '@/components/DataTable.vue'
import AvatarInitial from '@/components/AvatarInitial.vue'
import StatusBadge from '@/components/StatusBadge.vue'
import IconPlus from '@/components/icons/IconPlus.vue'
import IconX from '@/components/icons/IconX.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconDotsHorizontal from '@/components/icons/IconDotsHorizontal.vue'
import IconTrash from '@/components/icons/IconTrash.vue'
import IconKey from '@/components/icons/IconKey.vue'
import IconBan from '@/components/icons/IconBan.vue'
import IconEdit from '@/components/icons/IconEdit.vue'
import IconChevronLeft from '@/components/icons/IconChevronLeft.vue'
import IconChevronRight from '@/components/icons/IconChevronRight.vue'


const api = useApi()
const queryClient = useQueryClient()
const authStore = useAuthStore()
const { showConfirmModal, confirmTitle, confirmDescription, confirmLabel, confirmVariant, confirmCallback, openConfirm } = useConfirm()

// State
const page = ref(1)
const pageSize = ref(20)
const search = ref('')
const searchDebounced = refDebounced(search, 300)
const roleFilter = ref<string | undefined>(undefined)

const showCreateModal = ref(false)
const showDisableConfirm = ref(false)
const showDeleteConfirm = ref(false)
const showChangeRoleModal = ref(false)
const showResetPasswordModal = ref(false)

const userToDisable = ref<User | null>(null)
const userToDelete = ref<User | null>(null)
const userToChangeRole = ref<User | null>(null)
const userToResetPassword = ref<User | null>(null)

// Form state
const username = ref('')
const password = ref('')
const email = ref('')
const role = ref('manager')
const suspendReason = ref('')
const newRole = ref('')
const newPassword = ref('')
const forcePasswordChange = ref(true)


// Fetch users
const { data: usersData, isLoading, error, refetch } = useQuery({
  queryKey: ['users', page, pageSize, searchDebounced, roleFilter],
  queryFn: () => api.getUsers({
    page: page.value,
    page_size: pageSize.value,
    search: searchDebounced.value || undefined,
    role: roleFilter.value === 'all' ? undefined : roleFilter.value,
  }),
})

// Create user mutation
const createMutation = useMutation({
  mutationFn: (data: CreateUserRequest) => api.createUser(data),
  onSuccess: () => {
    showCreateModal.value = false
    username.value = ''
    password.value = ''
    email.value = ''
    role.value = 'manager'
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create user')
  },
})

function handleCreateUser() {
  if (!username.value.trim()) { toast.error('Username is required'); return }
  if (!email.value.trim()) { toast.error('Email is required'); return }
  if (!password.value) { toast.error('Password is required'); return }
  createMutation.mutate({
    username: username.value,
    password: password.value,
    email: email.value,
    role: role.value,
  })
}

// Suspend/Activate mutation
const suspendMutation = useMutation({
  mutationFn: ({ id, reason }: { id: string; reason?: string }) => api.suspendUser(id, reason),
  onSuccess: () => {
    showDisableConfirm.value = false
    userToDisable.value = null
    suspendReason.value = ''
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to suspend user')
  },
})

const activateMutation = useMutation({
  mutationFn: (id: string) => api.activateUser(id),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to activate user')
  },
})

function handleToggleStatus(user: User) {
  if (user.is_active) {
    userToDisable.value = user
    showDisableConfirm.value = true
  } else {
    openConfirm(
      'Activate User',
      `Restore access for ${user.username}?`,
      'Activate',
      () => activateMutation.mutate(user.id),
    )
  }
}

function confirmDisable() {
  if (userToDisable.value) {
    suspendMutation.mutate({ id: userToDisable.value.id, reason: suspendReason.value })
  }
}

// Delete user mutation
const deleteMutation = useMutation({
  mutationFn: (id: string) => api.deleteUser(id),
  onSuccess: () => {
    showDeleteConfirm.value = false
    userToDelete.value = null
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to delete user')
  },
})

function handleDeleteUser(user: User) {
  userToDelete.value = user
  showDeleteConfirm.value = true
}

function confirmDelete() {
  if (userToDelete.value) {
    deleteMutation.mutate(userToDelete.value.id)
  }
}

// Change role mutation
const changeRoleMutation = useMutation({
  mutationFn: ({ id, role }: { id: string; role: string }) => api.changeUserRole(id, role),
  onSuccess: () => {
    showChangeRoleModal.value = false
    userToChangeRole.value = null
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to change role')
  },
})

function handleChangeRole(user: User) {
  userToChangeRole.value = user
  newRole.value = user.role
  showChangeRoleModal.value = true
}

function confirmChangeRole() {
  if (userToChangeRole.value) {
    changeRoleMutation.mutate({ id: userToChangeRole.value.id, role: newRole.value })
  }
}

// Reset password mutation
const resetPasswordMutation = useMutation({
  mutationFn: ({ id, password, force }: { id: string; password: string; force: boolean }) =>
    api.resetUserPassword(id, password, force),
  onSuccess: () => {
    showResetPasswordModal.value = false
    userToResetPassword.value = null
    newPassword.value = ''
    forcePasswordChange.value = true
    toast.success('Password reset successfully')
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to reset password')
  },
})

function handleResetPassword(user: User) {
  userToResetPassword.value = user
  newPassword.value = ''
  forcePasswordChange.value = true
  showResetPasswordModal.value = true
}

function confirmResetPassword() {
  if (!newPassword.value) { toast.error('New password is required'); return }
  if (userToResetPassword.value) {
    resetPasswordMutation.mutate({
      id: userToResetPassword.value.id,
      password: newPassword.value,
      force: forcePasswordChange.value,
    })
  }
}

// Watch for filter changes to reset page
watch([searchDebounced, roleFilter], () => {
  page.value = 1
})

// Clear sensitive data when create dialog closes (e.g. via overlay/X)
watch(showCreateModal, (open) => {
  if (!open) { username.value = ''; password.value = ''; email.value = ''; role.value = 'manager' }
})



</script>

<template>
  <PageLayout title="User Management">
    <template #actions>

      <div class="w-full md:w-64">
        <Input id="search" v-model="search" placeholder="Search by username or email..." />
      </div>
      
      <div class="w-full md:w-40">
        <Select v-model="roleFilter">
          <SelectTrigger id="role-filter">
            <SelectValue placeholder="All Roles" class="text-foreground"/>
          </SelectTrigger>
          <SelectContent class="max-w-[200px]">
            <SelectItem value="all">All Roles</SelectItem>
            <SelectItem value="admin">Admin</SelectItem>
            <SelectItem value="manager">Manager</SelectItem>
            <SelectItem value="developer">Developer</SelectItem>
          </SelectContent>
        </Select>
      </div>


      <Button @click="showCreateModal = true">
        <IconPlus />
        <span class="max-sm:sr-only">Create User</span>
      </Button>
    </template>

    <DataTable
      :columns="[
        { label: 'User' },
        { label: 'Role' },
        { label: 'Status' },
        { label: 'Created' },
        { label: 'Actions', align: 'right' },
      ]"
      :is-loading="isLoading"
      :error="error"
      :is-empty="!usersData || usersData.users.length === 0"
      loading-text="Loading users..."
      :on-retry="() => refetch()"
    >
      <template #empty>
        <p class="text-muted-foreground mb-4">No users found</p>
        <Button @click="showCreateModal = true"><IconPlus />Create User</Button>
      </template>

      <tr v-for="user in usersData?.users" :key="user.id">
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap">
          <div class="flex gap-2 items-center">
            <AvatarInitial :name="user.username || 'u'" />
          <div class="flex flex-col min-w-0">
            <span class="text-base font-medium text-foreground max-w-[280px] truncate" :title="user.username">{{ user.username }}</span>
            <span class="text-muted-foreground text-xs max-w-[280px] truncate" :title="user.email || ''">{{ user.email }}</span>
          </div>
          </div>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">
          <Badge variant="outline">{{ user.role.charAt(0).toUpperCase() + user.role.slice(1) }}</Badge>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap">
          <StatusBadge :active="user.is_active" active-label="Active" inactive-label="Suspended" />
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">
          {{ formatTimestamp(user.created_at) }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-right text-base font-medium">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm">
                <span class="sr-only">Open menu</span>
                <IconDotsHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" class="max-w-[220px]">
              <DropdownMenuItem :disabled="user.id === authStore.userId" @click="handleChangeRole(user)">
                <IconEdit />
                Change Role
              </DropdownMenuItem>
              <DropdownMenuItem @click="handleResetPassword(user)">
                <IconKey />
                Reset Password
              </DropdownMenuItem>
              <DropdownMenuItem :disabled="user.id === authStore.userId" @click="handleToggleStatus(user)">
                <IconBan v-if="user.is_active" />
                <IconCheck v-else />
                {{ user.is_active ? 'Suspend' : 'Activate' }}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem :disabled="user.id === authStore.userId" class="text-destructive" @click="handleDeleteUser(user)">
                <IconTrash />
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </td>
      </tr>

      <template #footer>
        <div v-if="usersData" class="px-4 py-3 flex items-center justify-between border-t border-border sm:px-6">
          <p class="text-xs text-muted-foreground">
            <template v-if="usersData.total > 0">
              Showing <span class="font-medium">{{ (page - 1) * pageSize + 1 }}</span>
              to <span class="font-medium">{{ Math.min(page * pageSize, usersData.total) }}</span>
              of <span class="font-medium">{{ usersData.total }}</span> results
            </template>
            <template v-else>No results</template>
          </p>
          <div class="flex gap-2">
            <Button variant="outline" :disabled="page === 1" @click="page--"><IconChevronLeft />Previous</Button>
            <Button variant="outline" :disabled="page * pageSize >= usersData.total" @click="page++">Next<IconChevronRight /></Button>
          </div>
        </div>
      </template>
    </DataTable>

    <!-- Create user modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create New User</DialogTitle>
          <DialogDescription>
            Create a new user account.
          </DialogDescription>
        </DialogHeader>


        <div class="space-y-4">
          <div class="space-y-1.5">
            <Label for="username">Username</Label>
            <Input
              id="username"
              v-model="username"
              placeholder="username"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="email">Email</Label>
            <Input
              id="email"
              v-model="email"
              type="email"
              placeholder="user@example.com"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="password">Password</Label>
            <Input
              id="password"
              v-model="password"
              type="password"
              placeholder="password"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="role">Role</Label>
            <Select v-model="role">
              <SelectTrigger id="role">
                <SelectValue placeholder="Select a role" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="admin">Admin</SelectItem>
                <SelectItem value="manager">Manager</SelectItem>
                <SelectItem value="developer">Developer</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <DialogFooter>
          <Button
            :disabled="createMutation.isPending.value"
            variant="outline"
            @click="showCreateModal = false; username = ''; password = ''; email = ''; role = 'manager'"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="createMutation.isPending.value"
            @click="handleCreateUser"
          >
            <IconCheck />
            {{ createMutation.isPending.value ? 'Creating...' : 'Create User' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Suspend confirmation modal -->
    <Dialog v-model:open="showDisableConfirm">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Suspend User</DialogTitle>
          <DialogDescription>
            Are you sure you want to suspend user <strong>{{ userToDisable?.username }}</strong>?
          </DialogDescription>
        </DialogHeader>


        <div class="space-y-2">
          <Label for="reason">Reason (Optional)</Label>
          <Input
            id="reason"
            v-model="suspendReason"
            placeholder="Violation of terms..."
            :disabled="suspendMutation.isPending.value"
          />
        </div>

        <DialogFooter>
          <Button
            :disabled="suspendMutation.isPending.value"
            variant="outline"
            @click="showDisableConfirm = false"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="suspendMutation.isPending.value"
            variant="destructive"
            @click="confirmDisable"
          >
            <IconBan />
            {{ suspendMutation.isPending.value ? 'Suspending...' : 'Suspend User' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Delete confirmation modal -->
    <Dialog v-model:open="showDeleteConfirm">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Delete User</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete user <strong>{{ userToDelete?.username }}</strong>?
            This action cannot be undone.
          </DialogDescription>
        </DialogHeader>


        <DialogFooter>
          <Button
            :disabled="deleteMutation.isPending.value"
            variant="outline"
            @click="showDeleteConfirm = false"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="deleteMutation.isPending.value"
            variant="destructive"
            @click="confirmDelete"
          >
            <IconTrash />
            {{ deleteMutation.isPending.value ? 'Deleting...' : 'Delete User' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Change Role Modal -->
    <Dialog v-model:open="showChangeRoleModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Change User Role</DialogTitle>
          <DialogDescription>
            Change role for user <strong>{{ userToChangeRole?.username }}</strong>.
          </DialogDescription>
        </DialogHeader>


        <div class="space-y-2">
          <Label for="new-role">Role</Label>
          <Select v-model="newRole">
            <SelectTrigger id="new-role">
              <SelectValue placeholder="Select a role" />
            </SelectTrigger>
            <SelectContent class="max-w-[200px]">
              <SelectItem value="admin">Admin</SelectItem>
              <SelectItem value="manager">Manager</SelectItem>
              <SelectItem value="developer">Developer</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <DialogFooter>
          <Button
            :disabled="changeRoleMutation.isPending.value"
            variant="outline"
            @click="showChangeRoleModal = false"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="changeRoleMutation.isPending.value"
            @click="confirmChangeRole"
          >
            <IconCheck />
            {{ changeRoleMutation.isPending.value ? 'Saving...' : 'Save Changes' }}
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

    <!-- Reset Password Modal -->
    <Dialog v-model:open="showResetPasswordModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Reset Password</DialogTitle>
          <DialogDescription>
            Reset password for user <strong>{{ userToResetPassword?.username }}</strong>.
          </DialogDescription>
        </DialogHeader>


        <div class="space-y-4">
          <div class="space-y-1.5">
            <Label for="new-password">New Password</Label>
            <Input
              id="new-password"
              v-model="newPassword"
              type="password"
              placeholder="New secure password"
              :disabled="resetPasswordMutation.isPending.value"
            />
          </div>
          
          <div class="flex items-center space-x-2">
            <Switch
              id="force-change"
              v-model="forcePasswordChange"
              :disabled="resetPasswordMutation.isPending.value"
            />
            <Label for="force-change">Force password change on next login</Label>
          </div>
        </div>

        <DialogFooter>
          <Button
            :disabled="resetPasswordMutation.isPending.value"
            variant="outline"
            @click="showResetPasswordModal = false"
          >
            <IconX />
            Cancel
          </Button>
          <Button
            :disabled="resetPasswordMutation.isPending.value"
            @click="confirmResetPassword"
          >
            <IconKey />
            {{ resetPasswordMutation.isPending.value ? 'Resetting...' : 'Reset Password' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </PageLayout>
</template>
