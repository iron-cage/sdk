<script setup lang="ts">
import { ref } from 'vue'
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
import { Alert, AlertDescription } from '@/components/ui/alert'

const api = useApi()
const queryClient = useQueryClient()

const showCreateModal = ref(false)
const showDisableConfirm = ref(false)
const showDeleteConfirm = ref(false)
const userToDisable = ref<User | null>(null)
const userToDelete = ref<User | null>(null)
const username = ref('')
const password = ref('')
const role = ref('user')
const createError = ref('')
const updateError = ref('')
const deleteError = ref('')

// Fetch users
const { data: users, isLoading, error, refetch } = useQuery({
  queryKey: ['users'],
  queryFn: () => api.getUsers(),
})

// Create user mutation
const createMutation = useMutation({
  mutationFn: (data: CreateUserRequest) => api.createUser(data),
  onSuccess: () => {
    showCreateModal.value = false
    username.value = ''
    password.value = ''
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
    role: 'user',
  })
}

// Update status mutation
const updateStatusMutation = useMutation({
  mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
    api.updateUserStatus(id, isActive),
  onSuccess: () => {
    showDisableConfirm.value = false
    userToDisable.value = null
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
  onError: (err) => {
    updateError.value = err instanceof Error ? err.message : 'Failed to update status'
  },
})

function handleToggleStatus(user: User) {
  if (user.is_active) {
    // If active, show confirmation to disable
    userToDisable.value = user
    showDisableConfirm.value = true
  } else {
    // If inactive, enable immediately
    updateStatusMutation.mutate({ id: user.id, isActive: true })
  }
}

function confirmDisable() {
  if (userToDisable.value) {
    updateStatusMutation.mutate({ id: userToDisable.value.id, isActive: false })
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
</script>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-gray-900">User Management</h1>
      <Button @click="showCreateModal = true">
        Create New User
      </Button>
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
    <div v-else-if="users && users.length > 0" class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Username
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Role
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          <tr v-for="user in users" :key="user.id">
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ user.id }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ user.username }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              <Badge variant="outline">{{ user.role }}</Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <Badge :variant="user.is_active ? 'default' : 'destructive'">
                {{ user.is_active ? 'Active' : 'Inactive' }}
              </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
              <Button
                variant="ghost"
                size="sm"
                :class="user.is_active ? 'text-red-600 hover:text-red-900' : 'text-green-600 hover:text-green-900'"
                @click="handleToggleStatus(user)"
                :disabled="user.username === 'admin'"
              >
                {{ user.is_active ? 'Disable' : 'Enable' }}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                class="text-red-600 hover:text-red-900 ml-2"
                @click="handleDeleteUser(user)"
                :disabled="user.username === 'admin'"
              >
                Delete
              </Button>
            </td>
          </tr>
        </tbody>
      </table>
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
            Create a new user with username and password.
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
            <Label for="password">Password</Label>
            <Input
              id="password"
              type="password"
              v-model="password"
              placeholder="password"
              :disabled="createMutation.isPending.value"
            />
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

    <!-- Disable confirmation modal -->
    <Dialog v-model:open="showDisableConfirm">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Disable User</DialogTitle>
          <DialogDescription>
            Are you sure you want to disable user <strong>{{ userToDisable?.username }}</strong>?
            They will not be able to access the dashboard until re-enabled.
          </DialogDescription>
        </DialogHeader>

        <Alert v-if="updateError" variant="destructive" class="mb-4">
          <AlertDescription>{{ updateError }}</AlertDescription>
        </Alert>

        <DialogFooter>
          <Button
            @click="showDisableConfirm = false"
            :disabled="updateStatusMutation.isPending.value"
            variant="outline"
          >
            Cancel
          </Button>
          <Button
            @click="confirmDisable"
            :disabled="updateStatusMutation.isPending.value"
            variant="destructive"
          >
            {{ updateStatusMutation.isPending.value ? 'Disabling...' : 'Disable User' }}
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
  </div>
</template>
