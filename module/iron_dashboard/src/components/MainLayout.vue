<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const authStore = useAuthStore()
const sidebarOpen = ref(true)

async function handleLogout() {
  await authStore.logout()
  router.push('/login')
}
</script>

<template>
  <div class="min-h-screen bg-gray-100">
    <!-- Sidebar -->
    <div
      :class="[
        'fixed inset-y-0 left-0 z-50 w-64 bg-gray-900 transform transition-transform duration-300 ease-in-out',
        sidebarOpen ? 'translate-x-0' : '-translate-x-full',
      ]"
    >
      <div class="flex items-center justify-between h-16 px-4 bg-gray-800">
        <span class="text-xl font-semibold text-white">Iron Token</span>
        <button
          @click="sidebarOpen = !sidebarOpen"
          class="text-gray-400 hover:text-white focus:outline-none lg:hidden"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <nav class="px-4 py-6 space-y-2">
        <router-link
          to="/dashboard"
          class="flex items-center px-4 py-2 text-sm font-medium text-gray-300 rounded-md hover:bg-gray-800 hover:text-white"
        >
          <svg
            class="w-5 h-5 mr-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
            />
          </svg>
          Dashboard
        </router-link>

        <router-link
          to="/tokens"
          class="flex items-center px-4 py-2 text-sm font-medium text-gray-300 rounded-md hover:bg-gray-800 hover:text-white"
        >
          <svg
            class="w-5 h-5 mr-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
            />
          </svg>
          Tokens
        </router-link>

        <router-link
          to="/usage"
          class="flex items-center px-4 py-2 text-sm font-medium text-gray-300 rounded-md hover:bg-gray-800 hover:text-white"
        >
          <svg
            class="w-5 h-5 mr-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
            />
          </svg>
          Usage Analytics
        </router-link>

        <router-link
          to="/limits"
          class="flex items-center px-4 py-2 text-sm font-medium text-gray-300 rounded-md hover:bg-gray-800 hover:text-white"
        >
          <svg
            class="w-5 h-5 mr-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
            />
          </svg>
          Limits
        </router-link>

        <router-link
          to="/traces"
          class="flex items-center px-4 py-2 text-sm font-medium text-gray-300 rounded-md hover:bg-gray-800 hover:text-white"
        >
          <svg
            class="w-5 h-5 mr-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
          Traces
        </router-link>

        <router-link
          to="/providers"
          class="flex items-center px-4 py-2 text-sm font-medium text-gray-300 rounded-md hover:bg-gray-800 hover:text-white"
        >
          <svg
            class="w-5 h-5 mr-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"
            />
          </svg>
          AI Providers
        </router-link>

        <router-link
          v-if="authStore.isAdmin"
          to="/users"
          class="flex items-center px-4 py-2 text-sm font-medium text-gray-300 rounded-md hover:bg-gray-800 hover:text-white"
        >
          <svg
            class="w-5 h-5 mr-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
            />
          </svg>
          Users
        </router-link>
      </nav>
    </div>

    <!-- Main content -->
    <div :class="['transition-all duration-300', sidebarOpen ? 'lg:ml-64' : '']">
      <!-- Header -->
      <header class="bg-white shadow-sm">
        <div class="flex items-center justify-between h-16 px-4">
          <button
            @click="sidebarOpen = !sidebarOpen"
            class="text-gray-500 hover:text-gray-700 focus:outline-none"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 12h16M4 18h16"
              />
            </svg>
          </button>

          <div class="flex items-center space-x-4">
            <span class="text-sm text-gray-700">{{ authStore.username }}</span>
            <button
              @click="handleLogout"
              class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 focus:outline-none"
            >
              Logout
            </button>
          </div>
        </div>
      </header>

      <!-- Page content -->
      <main class="p-6">
        <router-view />
      </main>
    </div>
  </div>
</template>
