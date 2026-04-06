<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'

import { useAuthStore } from '@/stores/auth'
import { cn } from '@/lib/utils'

import AvatarInitial from './AvatarInitial.vue'
import IconX from './icons/IconX.vue'
import IconHome from './icons/IconHome.vue'
import IconChevronDown from './icons/IconChevronDown.vue'
import IconCog from './icons/IconCog.vue'
import IconBarChart from './icons/IconBarChart.vue'
import IconCoin from './icons/IconCoin.vue'
import IconChip from './icons/IconChip.vue'
import IconUsers from './icons/IconUsers.vue'
import IconLogOut from './icons/IconLogOut.vue'
import IconMenu from './icons/IconMenu.vue'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const sidebarOpen = ref(false)
const workspaceOpen = ref(true)
const adminOpen = ref(true)
const userMenuOpen = ref(false)
const userMenuRef = ref<HTMLElement | null>(null)

function onClickOutside(e: MouseEvent) {
  if (userMenuOpen.value && userMenuRef.value && !userMenuRef.value.contains(e.target as Node)) {
    userMenuOpen.value = false
  }
}
onMounted(() => document.addEventListener('click', onClickOutside, true))
onUnmounted(() => document.removeEventListener('click', onClickOutside, true))

function isActive(path: string): boolean {
  return route.path === path || route.path.startsWith(path + '/')
}

function navLinkClass(path: string): string {
  return cn(
    'flex items-center px-3 py-1.5 text-base rounded-[8px] text-tertiary',
    isActive(path) ? 'bg-border/80 font-medium' : 'hover:bg-border/50',
  )
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
          aria-label="Close navigation menu"
          @click="sidebarOpen = false"
        >
          <IconX />
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
            <IconHome class="w-4 h-4 mr-2 flex-shrink-0" />
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
            <IconChevronDown class="w-3 h-3 transition-transform duration-150" :class="workspaceOpen ? '' : '-rotate-90'" />
          </button>
          <div v-show="workspaceOpen" class="mt-0.5 space-y-0.5 ">
            <router-link
              to="/agents"
              :class="navLinkClass('/agents')"
              @click="handleNavClick"
            >
              <IconCog class="w-4 h-4 mr-2 flex-shrink-0" />
              Agents
            </router-link>

            <router-link
              to="/usage"
              :class="navLinkClass('/usage')"
              @click="handleNavClick"
            >
              <IconBarChart class="w-4 h-4 mr-2 flex-shrink-0" />
              Analytics
            </router-link>

            <router-link
              to="/budgets"
              :class="navLinkClass('/budgets')"
              @click="handleNavClick"
            >
              <IconCoin class="w-4 h-4 mr-2 flex-shrink-0" />
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
            <IconChevronDown class="w-3 h-3 transition-transform duration-150" :class="adminOpen ? '' : '-rotate-90'" />
          </button>
          <div v-show="adminOpen" class="mt-0.5 space-y-0.5 ">
            <router-link
              to="/providers"
              :class="navLinkClass('/providers')"
              @click="handleNavClick"
            >
              <IconChip class="w-4 h-4 mr-2 flex-shrink-0" />
              Providers
            </router-link>

            <router-link
              to="/users"
              :class="navLinkClass('/users')"
              @click="handleNavClick"
            >
              <IconUsers class="w-4 h-4 mr-2 flex-shrink-0" />
              Users
            </router-link>
          </div>
        </div>
      </nav>

      <!-- Sidebar footer -->
      <div ref="userMenuRef" class="p-2 relative">
        <!-- Dropdown menu -->
        <Transition
          enter-active-class="transition duration-150 ease-out"
          enter-from-class="opacity-0 translate-y-1"
          enter-to-class="opacity-100 translate-y-0"
          leave-active-class="transition duration-150 ease-in"
          leave-from-class="opacity-100 translate-y-0"
          leave-to-class="opacity-0 translate-y-1"
        >
        <div
          v-if="userMenuOpen"
          class="absolute bottom-full left-2 right-2 mb-1 bg-background border border-border rounded-[8px] shadow-md overflow-hidden"
        >
          <button
            class="flex items-center gap-2 w-full px-3 py-2 text-base text-tertiary hover:bg-border/50 text-left"
            @click="handleLogout"
          >
            <IconLogOut class="w-4 h-4 flex-shrink-0" />
            Sign out
          </button>
        </div>
        </Transition>

        <!-- Trigger button -->
        <button
          class="flex items-center gap-2 w-full px-2 py-1.5 rounded-[8px] hover:bg-border/50 text-left"
          @click="userMenuOpen = !userMenuOpen"
        >
          <!-- Avatar -->
          <AvatarInitial :name="authStore.username || 'u'" />
          <!-- Name -->
          <span class="text-base text-tertiary truncate flex-1">{{ authStore.username }}</span>
          <!-- Chevron -->
          <IconChevronDown class="w-3 h-3 text-muted-foreground transition-transform duration-150 flex-shrink-0" :class="userMenuOpen ? 'rotate-180' : ''" />
        </button>
      </div>
    </div>

    <!-- Main content -->
    <div class="lg:ml-56 lg:my-[8px] lg:mr-[8px] rounded-[8px] h-screen lg:h-[calc(100vh-16px)] border border-border overflow-hidden bg-background flex flex-col">
      <!-- Mobile header -->
      <div class="lg:hidden h-12 flex items-center px-3 border-b border-border shrink-0">
        <button
          class="p-1 rounded text-muted-foreground hover:text-foreground"
          aria-label="Open navigation menu"
          @click="sidebarOpen = true"
        >
          <IconMenu />
        </button>
      </div>

      <!-- Page content -->
      <main class="flex-1 overflow-hidden">
        <router-view />
      </main>
    </div>
  </div>
</template>
