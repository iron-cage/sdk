<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const sidebarOpen = ref(false)
const workspaceOpen = ref(true)
const adminOpen = ref(true)
const userMenuOpen = ref(false)

const AVATAR_COLORS = ['#5E6AD2', '#26B5CE', '#4CB782', '#F2994A']

function avatarColor(name: string): string {
  let hash = 0
  for (let i = 0; i < name.length; i++) hash += name.charCodeAt(i)
  return AVATAR_COLORS[hash % AVATAR_COLORS.length]!
}

function isActive(path: string): boolean {
  return route.path === path || route.path.startsWith(path + '/')
}

function navLinkClass(path: string): string {
  let base = 'flex items-center px-3 py-1.5 text-base rounded-[8px]'
  
  if (isActive(path)) {
    return `${base} bg-border/80 text-trettiary font-medium`
  }
  return `${base} text-trettiary hover:bg-border/50`
}

function handleNavClick() {
  sidebarOpen.value = false
}

async function handleLogout() {
  await authStore.logout()
  router.push('/login')
}
</script>

<template>
  <div class="min-h-screen bg-muted">
    <!-- Mobile sidebar backdrop -->
    <div
      v-if="sidebarOpen"
      class="fixed inset-0 z-40 bg-black/50 lg:hidden"
      @click="sidebarOpen = false"
    />

    <!-- Sidebar -->
    <div
      class="fixed inset-y-0 left-0 z-50 w-56 bg-muted flex flex-col transform transition-transform duration-200 ease-in-out lg:translate-x-0"
      :class="sidebarOpen ? 'translate-x-0' : '-translate-x-full'"
    >
      <!-- Logo row -->
      <div class="h-4 mt-4 flex items-center justify-between px-3">
        <span class="text-md font-semibold text-foreground">Iron Cage</span>
        <button
          class="lg:hidden text-foreground"
          @click="sidebarOpen = false"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <nav class="px-2 py-3 flex-1 overflow-y-auto space-y-3">
        <!-- Dashboard -->
        <div>
          <router-link
            to="/dashboard"
            :class="navLinkClass('/dashboard')"
            @click="handleNavClick"
          >
            <svg class="w-4 h-4 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
            </svg>
            Dashboard
          </router-link>
        </div>

        <!-- Workspace group -->
        <div>
          <button
            class="flex items-center gap-2 w-full px-3 py-1 text-xs font-medium tracking-wider hover:text-foreground opacity-70"
            @click="workspaceOpen = !workspaceOpen"
          >
            <span>Workspace</span>
            <svg
              class="w-3 h-3 transition-transform duration-150"
              :class="workspaceOpen ? '' : '-rotate-90'"
              fill="none" stroke="currentColor" viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div v-show="workspaceOpen" class="mt-0.5 space-y-0.5 ">
            <router-link
              to="/agents"
              :class="navLinkClass('/agents')"
              @click="handleNavClick"
            >
              <svg class="w-4 h-4 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
              Agents
            </router-link>

            <router-link
              to="/usage"
              :class="navLinkClass('/usage')"
              @click="handleNavClick"
            >
              <svg class="w-4 h-4 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              Analytics
            </router-link>

            <router-link
              to="/limits"
              :class="navLinkClass('/limits')"
              @click="handleNavClick"
            >
              <svg class="w-4 h-4 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Budgets
            </router-link>
          </div>
        </div>

        <!-- Admin group -->
        <div v-if="authStore.isAdmin">
          <button
              class="flex items-center gap-2 w-full px-3 py-1 text-xs font-medium tracking-wider hover:text-foreground opacity-70"           @click="adminOpen = !adminOpen"
          >
            <span>Admin</span>
            <svg
              class="w-3 h-3 transition-transform duration-150"
              :class="adminOpen ? '' : '-rotate-90'"
              fill="none" stroke="currentColor" viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div v-show="adminOpen" class="mt-0.5 space-y-0.5 ">
            <router-link
              to="/providers"
              :class="navLinkClass('/providers')"
              @click="handleNavClick"
            >
              <svg class="w-4 h-4 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
              Providers
            </router-link>

            <router-link
              to="/users"
              :class="navLinkClass('/users')"
              @click="handleNavClick"
            >
              <svg class="w-4 h-4 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
              Users
            </router-link>
          </div>
        </div>
      </nav>

      <!-- Sidebar footer -->
      <div class="p-2 relative">
        <!-- Dropdown menu -->
        <div
          v-if="userMenuOpen"
          class="absolute bottom-full left-2 right-2 mb-1 bg-background border border-border rounded-[8px] shadow-md overflow-hidden"
        >
          <button
            @click="handleLogout"
            class="flex items-center gap-2 w-full px-3 py-2 text-base text-trettiary hover:bg-border/50 text-left"
          >
            <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
            </svg>
            Sign out
          </button>
        </div>

        <!-- Trigger button -->
        <button
          @click="userMenuOpen = !userMenuOpen"
          class="flex items-center gap-2 w-full px-2 py-1.5 rounded-[8px] hover:bg-border/50 text-left"
        >
          <!-- Avatar -->
          <span
            class="w-6 h-6 rounded-[6px] flex items-center justify-center text-xs font-semibold text-white flex-shrink-0"
            :style="{ backgroundColor: avatarColor(authStore.username || 'u') }"
          >
            {{ (authStore?.username ?? 'U')[0]!.toUpperCase() }}
          </span>
          <!-- Name -->
          <span class="text-base text-trettiary truncate flex-1">{{ authStore.username }}</span>
          <!-- Chevron -->
          <svg
            class="w-3 h-3 text-muted-foreground transition-transform duration-150 flex-shrink-0"
            :class="userMenuOpen ? 'rotate-180' : ''"
            fill="none" stroke="currentColor" viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Main content -->
    <div class="lg:ml-56 lg:my-[8px] lg:mr-[8px] rounded-[8px] h-[calc(100vh-16px)] border border-border overflow-hidden bg-background flex flex-col">
      <!-- Mobile header -->
      <div class="lg:hidden h-12 flex items-center px-3 border-b border-border shrink-0">
        <button
          class="p-1 rounded text-muted-foreground hover:text-foreground"
          @click="sidebarOpen = true"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>
      </div>

      <!-- Page content -->
      <main class="flex-1 overflow-hidden">
        <router-view />
      </main>
    </div>
  </div>
</template>
