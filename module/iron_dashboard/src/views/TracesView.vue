<script setup lang="ts">
import { ref, computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '../composables/useApi'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

const api = useApi()

// Filters
const filterTokenId = ref('')
const filterProvider = ref('__all__')
const filterModel = ref('__all__')

// Fetch traces
const { data: traces, isLoading, error, refetch } = useQuery({
  queryKey: ['traces'],
  queryFn: () => api.getTraces(),
})

// Filtered traces
const filteredTraces = computed(() => {
  if( !traces.value ) return []

  return traces.value.filter( ( trace ) => {
    if( filterTokenId.value && !String( trace.token_id ).includes( filterTokenId.value ) ) {
      return false
    }
    if( filterProvider.value && filterProvider.value !== '__all__' && !trace.provider.toLowerCase().includes( filterProvider.value.toLowerCase() ) ) {
      return false
    }
    if( filterModel.value && filterModel.value !== '__all__' && !trace.model.toLowerCase().includes( filterModel.value.toLowerCase() ) ) {
      return false
    }
    return true
  })
})

// Unique providers and models for filter dropdowns
const uniqueProviders = computed(() => {
  if( !traces.value ) return []
  const providers = new Set( traces.value.map( t => t.provider ) )
  return Array.from( providers ).sort()
})

const uniqueModels = computed(() => {
  if( !traces.value ) return []
  const models = new Set( traces.value.map( t => t.model ) )
  return Array.from( models ).sort()
})

function formatDate( timestamp: number ): string {
  return new Date( timestamp ).toLocaleString()
}

function formatCost( cost: number ): string {
  return `$${cost.toFixed( 4 )}`
}

function formatNumber( num: number ): string {
  return num.toLocaleString()
}

function clearFilters() {
  filterTokenId.value = ''
  filterProvider.value = '__all__'
  filterModel.value = '__all__'
}
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 mb-6">API Call Traces</h1>

    <!-- Filters -->
    <Card class="mb-6">
      <CardHeader>
        <CardTitle>Filters</CardTitle>
      </CardHeader>
      <CardContent>
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div class="space-y-2">
            <Label for="filterTokenId">Token ID</Label>
            <Input
              id="filterTokenId"
              v-model="filterTokenId"
              placeholder="Filter by token ID"
            />
          </div>

          <div class="space-y-2">
            <Label for="filterProvider">Provider</Label>
            <Select v-model="filterProvider">
              <SelectTrigger id="filterProvider">
                <SelectValue placeholder="All Providers" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__all__">All Providers</SelectItem>
                <SelectItem v-for="provider in uniqueProviders" :key="provider" :value="provider">
                  {{ provider }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-2">
            <Label for="filterModel">Model</Label>
            <Select v-model="filterModel">
              <SelectTrigger id="filterModel">
                <SelectValue placeholder="All Models" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__all__">All Models</SelectItem>
                <SelectItem v-for="model in uniqueModels" :key="model" :value="model">
                  {{ model }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="flex items-end">
            <Button @click="clearFilters" variant="secondary" class="w-full">
              Clear Filters
            </Button>
          </div>
        </div>

        <div class="mt-4 text-sm text-gray-600">
          Showing {{ filteredTraces.length }} of {{ traces?.length || 0 }} traces
        </div>
      </CardContent>
    </Card>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading traces...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading traces: {{ error.message }}</p>
      <Button @click="() => refetch()" variant="secondary" class="mt-4">
        Retry
      </Button>
    </div>

    <!-- Traces table -->
    <div v-else-if="filteredTraces.length > 0" class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Timestamp
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Request ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Token ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Provider
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Model
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Input Tokens
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Output Tokens
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Cost
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          <tr v-for="trace in filteredTraces" :key="trace.id" class="hover:bg-gray-50">
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ formatDate( trace.timestamp ) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
              {{ trace.request_id.substring( 0, 8 ) }}...
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ trace.token_id }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ trace.provider }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ trace.model }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
              {{ formatNumber( trace.input_tokens ) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
              {{ formatNumber( trace.output_tokens ) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-right font-medium text-gray-900">
              {{ formatCost( trace.cost ) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else class="bg-white rounded-lg shadow p-6 text-center">
      <p class="text-gray-600">
        {{ traces?.length === 0 ? 'No traces found' : 'No traces match the current filters' }}
      </p>
      <Button
        v-if="filterTokenId || (filterProvider !== '__all__') || (filterModel !== '__all__')"
        @click="clearFilters"
        class="mt-4"
      >
        Clear Filters
      </Button>
    </div>
  </div>
</template>
