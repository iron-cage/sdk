<script setup lang="ts">
import { ref, watch } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi, type CreateUserRequest, type User } from '../composables/useApi'
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import PageLayout from '@/components/PageLayout.vue'
import DataTable from '@/components/DataTable.vue'


const api = useApi()
const queryClient = useQueryClient()

// State
const page = ref(1)
const pageSize = ref(20)
const search = ref('')
const roleFilter = ref<string | undefined>(undefined)
const isActiveFilter = ref<boolean | undefined>(undefined)

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
  queryKey: ['users', page, pageSize, search, roleFilter, isActiveFilter],
  queryFn: () => api.getUsers({
    page: page.value,
    page_size: pageSize.value,
    search: search.value || undefined,
    role: roleFilter.value === 'all' ? undefined : roleFilter.value,
    is_active: isActiveFilter.value
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
    role.value = 'user'
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create user')
  },
})

function handleCreateUser() {
  createMutation.mutate({
    username: username.value,
    password: password.value,
    email: email.value,
    role: role.value,
  })
}

// Suspend/Activate mutation
const suspendMutation = useMutation({
  mutationFn: ({ id, reason }: { id: number; reason?: string }) => api.suspendUser(id, reason),
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
  mutationFn: (id: number) => api.activateUser(id),
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
    activateMutation.mutate(user.id)
  }
}

function confirmDisable() {
  if (userToDisable.value) {
    suspendMutation.mutate({ id: userToDisable.value.id, reason: suspendReason.value })
  }
}

// Delete user mutation
const deleteMutation = useMutation({
  mutationFn: (id: number) => api.deleteUser(id),
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
  mutationFn: ({ id, role }: { id: number; role: string }) => api.changeUserRole(id, role),
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
  mutationFn: ({ id, password, force }: { id: number; password: string; force: boolean }) => 
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
  if (userToResetPassword.value) {
    resetPasswordMutation.mutate({ 
      id: userToResetPassword.value.id, 
      password: newPassword.value, 
      force: forcePasswordChange.value 
    })
  }
}

// Watch for search changes to reset page
watch(search, () => {
  page.value = 1
})


const AVATAR_COLORS = ['#5E6AD2', '#26B5CE', '#4CB782', '#F2994A']

function avatarColor(name: string): string {
  let hash = 0
  for (let i = 0; i < name.length; i++) hash += name.charCodeAt(i)
  return AVATAR_COLORS[hash % AVATAR_COLORS.length]!
}

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
            <SelectValue placeholder="All Roles" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Roles</SelectItem>
            <SelectItem value="admin">Admin</SelectItem>
            <SelectItem value="manager">Manager</SelectItem>
            <SelectItem value="developer">Developer</SelectItem>
          </SelectContent>
        </Select>
      </div>


      <Button @click="showCreateModal = true">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
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
        <Button @click="showCreateModal = true"><svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>Create First User</Button>
      </template>

      <tr v-for="user in usersData?.users" :key="user.id">
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap">
          <div class="flex gap-2 items-center">
            <span
            class="w-6 h-6 rounded-[6px] flex items-center justify-center text-xs font-semibold text-white flex-shrink-0"
            :style="{ backgroundColor: avatarColor(user.username || 'u') }"
          >
            {{ (user.username ?? 'U')[0]!.toUpperCase() }}
          </span>
          <div class="flex flex-col">
            <span class="text-base font-medium text-foreground">{{ user.username }}</span>
            <span class="text-muted-foreground text-xs">{{ user.email }}</span>
          </div>
          </div>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">
          <Badge variant="outline">{{ user.role.charAt(0).toUpperCase() + user.role.slice(1) }}</Badge>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap">
          <Badge variant='outline' class="text-foreground border-none">
            <svg width="7" height="7" viewBox="0 0 12 12" fill="none" :class ="user.is_active ? 'text-success' : 'text-destructive'"> <circle cx="6" cy="6" r="6" fill="currentColor" /> </svg> 
            {{ user.is_active ? 'Active' : 'Suspended' }}
          </Badge>
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">
          {{ new Date(user.created_at).toLocaleDateString() }}
        </td>
        <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-right text-base font-medium">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm">
                <span class="sr-only">Open menu</span>
                <svg width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4"><path d="M3.625 7.5C3.625 8.12132 3.12132 8.625 2.5 8.625C1.87868 8.625 1.375 8.12132 1.375 7.5C1.375 6.87868 1.87868 6.375 2.5 6.375C3.12132 6.375 3.625 6.87868 3.625 7.5ZM8.625 7.5C8.625 8.12132 8.12132 8.625 7.5 8.625C6.87868 8.625 6.375 8.12132 6.375 7.5C6.375 6.87868 6.87868 6.375 7.5 6.375C8.12132 6.375 8.625 6.87868 8.625 7.5ZM13.625 7.5C13.625 8.12132 13.1213 8.625 12.5 8.625C11.8787 8.625 11.375 8.12132 11.375 7.5C11.375 6.87868 11.8787 6.375 12.5 6.375C13.1213 6.375 13.625 6.87868 13.625 7.5Z" fill="currentColor" fill-rule="evenodd" clip-rule="evenodd"></path></svg>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem @click="handleChangeRole(user)" :disabled="user.username === 'admin'">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" /></svg>
                Change Role
              </DropdownMenuItem>
              <DropdownMenuItem @click="handleResetPassword(user)">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" /></svg>
                Reset Password
              </DropdownMenuItem>
              <DropdownMenuItem @click="handleToggleStatus(user)" :disabled="user.username === 'admin'">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="user.is_active ? 'M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z' : 'M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z M21 12a9 9 0 11-18 0 9 9 0 0118 0z'" /></svg>
                {{ user.is_active ? 'Suspend' : 'Activate' }}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem @click="handleDeleteUser(user)" :disabled="user.username === 'admin'" class="text-destructive">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </td>
      </tr>

      <template #footer>
        <div v-if="usersData" class="px-4 py-3 flex items-center justify-between border-t border-border sm:px-6">
          <p class="text-xs text-muted-foreground">
            Showing <span class="font-medium">{{ (page - 1) * pageSize + 1 }}</span>
            to <span class="font-medium">{{ Math.min(page * pageSize, usersData.total) }}</span>
            of <span class="font-medium">{{ usersData.total }}</span> results
          </p>
          <div class="flex gap-2">
            <Button variant="outline" :disabled="page === 1" @click="page--"><svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>Previous</Button>
            <Button variant="outline" :disabled="page * pageSize >= usersData.total" @click="page++">Next<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg></Button>
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
              type="email"
              v-model="email"
              placeholder="user@example.com"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="password">Password</Label>
            <Input
              id="password"
              type="password"
              v-model="password"
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
            @click="showCreateModal = false"
            :disabled="createMutation.isPending.value"
            variant="outline"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="handleCreateUser"
            :disabled="createMutation.isPending.value"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
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
            @click="showDisableConfirm = false"
            :disabled="suspendMutation.isPending.value"
            variant="outline"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="confirmDisable"
            :disabled="suspendMutation.isPending.value"
            variant="destructive"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
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
            @click="showDeleteConfirm = false"
            :disabled="deleteMutation.isPending.value"
            variant="outline"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="confirmDelete"
            :disabled="deleteMutation.isPending.value"
            variant="destructive"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
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
            <SelectContent>
              <SelectItem value="admin">Admin</SelectItem>
              <SelectItem value="manager">Manager</SelectItem>
              <SelectItem value="developer">Developer</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <DialogFooter>
          <Button
            @click="showChangeRoleModal = false"
            :disabled="changeRoleMutation.isPending.value"
            variant="outline"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="confirmChangeRole"
            :disabled="changeRoleMutation.isPending.value"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            {{ changeRoleMutation.isPending.value ? 'Saving...' : 'Save Changes' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

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
              type="password"
              v-model="newPassword"
              placeholder="New secure password"
              :disabled="resetPasswordMutation.isPending.value"
            />
          </div>
          
          <div class="flex items-center space-x-2">
            <input 
              id="force-change" 
              type="checkbox" 
              class="h-4 w-4 rounded border-border text-accent focus:ring-ring"
              :checked="forcePasswordChange" 
              @change="forcePasswordChange = ($event.target as HTMLInputElement).checked" 
            />
            <Label for="force-change">Force password change on next login</Label>
          </div>
        </div>

        <DialogFooter>
          <Button
            @click="showResetPasswordModal = false"
            :disabled="resetPasswordMutation.isPending.value"
            variant="outline"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            Cancel
          </Button>
          <Button
            @click="confirmResetPassword"
            :disabled="resetPasswordMutation.isPending.value"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" /></svg>
            {{ resetPasswordMutation.isPending.value ? 'Resetting...' : 'Reset Password' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </PageLayout>
</template>
