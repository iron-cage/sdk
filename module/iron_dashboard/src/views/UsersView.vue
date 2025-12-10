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
import { Alert, AlertDescription } from '@/components/ui/alert'


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
const role = ref('user')
const suspendReason = ref('')
const newRole = ref('')
const newPassword = ref('')
const forcePasswordChange = ref(true)

// Errors
const createError = ref('')
const updateError = ref('')
const deleteError = ref('')
const roleError = ref('')
const passwordError = ref('')

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
    createError.value = ''
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    createError.value = err instanceof Error ? err.message : 'Failed to create user'
  },
})

function handleCreateUser() {
  createError.value = ''
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
    updateError.value = err instanceof Error ? err.message : 'Failed to suspend user'
  },
})

const activateMutation = useMutation({
  mutationFn: (id: number) => api.activateUser(id),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    // Show toast or alert
    console.error(err)
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
    deleteError.value = err instanceof Error ? err.message : 'Failed to delete user'
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
    roleError.value = err instanceof Error ? err.message : 'Failed to change role'
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
    // Show success toast
  },
  onError: (err) => {
    passwordError.value = err instanceof Error ? err.message : 'Failed to reset password'
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
</script>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-gray-900">User Management</h1>
      <Button @click="showCreateModal = true">
        Create New User
      </Button>
    </div>

    <!-- Filters -->
    <div class="bg-white rounded-lg shadow p-4 mb-6 flex flex-wrap gap-4 items-end">
      <div class="w-full md:w-64">
        <Label for="search">Search</Label>
        <Input id="search" v-model="search" placeholder="Search by username or email..." />
      </div>
      
      <div class="w-full md:w-40">
        <Label for="role-filter">Role</Label>
        <Select v-model="roleFilter">
          <SelectTrigger id="role-filter">
            <SelectValue placeholder="All Roles" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Roles</SelectItem>
            <SelectItem value="admin">Admin</SelectItem>
            <SelectItem value="user">User</SelectItem>
            <SelectItem value="viewer">Viewer</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading users...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading users: {{ error.message }}</p>
      <Button @click="() => refetch()" variant="secondary" class="mt-4">
        Retry
      </Button>
    </div>

    <!-- Users table -->
    <div v-else-if="usersData && usersData.users.length > 0" class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              User
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Role
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Created
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          <tr v-for="user in usersData.users" :key="user.id">
            <td class="px-6 py-4 whitespace-nowrap">
              <div class="flex flex-col">
                <span class="text-sm font-medium text-gray-900">{{ user.username }}</span>
                <span class="text-sm text-gray-500">{{ user.email }}</span>
              </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              <Badge variant="outline">{{ user.role }}</Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <Badge :variant="user.is_active ? 'default' : 'destructive'">
                {{ user.is_active ? 'Active' : 'Suspended' }}
              </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ new Date(user.created_at).toLocaleDateString() }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
              <div class="flex justify-end gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  @click="handleChangeRole(user)"
                  :disabled="user.username === 'admin'"
                >
                  Role
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  @click="handleResetPassword(user)"
                >
                  Reset Pass
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  :class="user.is_active ? 'text-orange-600 hover:text-orange-900' : 'text-green-600 hover:text-green-900'"
                  @click="handleToggleStatus(user)"
                  :disabled="user.username === 'admin'"
                >
                  {{ user.is_active ? 'Suspend' : 'Activate' }}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  class="text-red-600 hover:text-red-900"
                  @click="handleDeleteUser(user)"
                  :disabled="user.username === 'admin'"
                >
                  Delete
                </Button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      
      <!-- Pagination -->
      <div class="bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 sm:px-6">
        <div class="flex-1 flex justify-between sm:hidden">
          <Button :disabled="page === 1" @click="page--">Previous</Button>
          <Button :disabled="page * pageSize >= usersData.total" @click="page++">Next</Button>
        </div>
        <div class="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
          <div>
            <p class="text-sm text-gray-700">
              Showing <span class="font-medium">{{ (page - 1) * pageSize + 1 }}</span> to <span class="font-medium">{{ Math.min(page * pageSize, usersData.total) }}</span> of <span class="font-medium">{{ usersData.total }}</span> results
            </p>
          </div>
          <div>
            <nav class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px" aria-label="Pagination">
              <Button 
                variant="outline" 
                :disabled="page === 1" 
                @click="page--"
                class="rounded-l-md"
              >
                Previous
              </Button>
              <Button 
                variant="outline" 
                :disabled="page * pageSize >= usersData.total" 
                @click="page++"
                class="rounded-r-md"
              >
                Next
              </Button>
            </nav>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="bg-white rounded-lg shadow p-6 text-center">
      <p class="text-gray-600 mb-4">No users found</p>
      <Button @click="showCreateModal = true">
        Create First User
      </Button>
    </div>

    <!-- Create user modal -->
    <Dialog v-model:open="showCreateModal">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create New User</DialogTitle>
          <DialogDescription>
            Create a new user account.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="createError" variant="destructive" class="mb-4">
          <AlertDescription>{{ createError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="username">Username</Label>
            <Input
              id="username"
              v-model="username"
              placeholder="username"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-2">
            <Label for="email">Email</Label>
            <Input
              id="email"
              type="email"
              v-model="email"
              placeholder="user@example.com"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-2">
            <Label for="password">Password</Label>
            <Input
              id="password"
              type="password"
              v-model="password"
              placeholder="password"
              :disabled="createMutation.isPending.value"
            />
          </div>

          <div class="space-y-2">
            <Label for="role">Role</Label>
            <Select v-model="role">
              <SelectTrigger id="role">
                <SelectValue placeholder="Select a role" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="admin">Admin</SelectItem>
                <SelectItem value="user">User</SelectItem>
                <SelectItem value="viewer">Viewer</SelectItem>
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
            Cancel
          </Button>
          <Button
            @click="handleCreateUser"
            :disabled="createMutation.isPending.value"
          >
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

        <Alert v-if="updateError" variant="destructive" class="mb-4">
          <AlertDescription>{{ updateError }}</AlertDescription>
        </Alert>

        <div class="space-y-2 py-2">
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
            Cancel
          </Button>
          <Button
            @click="confirmDisable"
            :disabled="suspendMutation.isPending.value"
            variant="destructive"
          >
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

        <Alert v-if="deleteError" variant="destructive" class="mb-4">
          <AlertDescription>{{ deleteError }}</AlertDescription>
        </Alert>

        <DialogFooter>
          <Button
            @click="showDeleteConfirm = false"
            :disabled="deleteMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="confirmDelete"
            :disabled="deleteMutation.isPending.value"
            variant="destructive"
          >
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

        <Alert v-if="roleError" variant="destructive" class="mb-4">
          <AlertDescription>{{ roleError }}</AlertDescription>
        </Alert>

        <div class="space-y-2 py-4">
          <Label for="new-role">Role</Label>
          <Select v-model="newRole">
            <SelectTrigger id="new-role">
              <SelectValue placeholder="Select a role" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="admin">Admin</SelectItem>
              <SelectItem value="user">User</SelectItem>
              <SelectItem value="viewer">Viewer</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <DialogFooter>
          <Button
            @click="showChangeRoleModal = false"
            :disabled="changeRoleMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="confirmChangeRole"
            :disabled="changeRoleMutation.isPending.value"
          >
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

        <Alert v-if="passwordError" variant="destructive" class="mb-4">
          <AlertDescription>{{ passwordError }}</AlertDescription>
        </Alert>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
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
              class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
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
            Cancel
          </Button>
          <Button
            @click="confirmResetPassword"
            :disabled="resetPasswordMutation.isPending.value"
          >
            {{ resetPasswordMutation.isPending.value ? 'Resetting...' : 'Reset Password' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
