<script setup lang="ts">
import type { VM } from '../types'

defineProps<{
  vms: VM[]
  loading: boolean
}>()

const emit = defineEmits<{
  select: [vm: VM]
  action: [vmName: string, action: 'start' | 'stop' | 'destroy']
}>()

function getStateColor(state: string): string {
  switch (state) {
    case 'running': return 'bg-green-500'
    case 'stopped': return 'bg-[#8c909f]'
    default: return 'bg-yellow-500'
  }
}

function formatMemory(mb: number): string {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)} GB`
  }
  return `${mb} MB`
}
</script>

<template>
  <div class="bg-[#171c21] rounded-lg border border-[#424753]/20 overflow-hidden">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-[#424753]/20 flex items-center justify-between">
      <h2 class="font-black tracking-tight text-lg">Virtual Machines</h2>
      <div class="flex items-center gap-2 text-xs text-[#8c909f]">
        <span class="w-2 h-2 rounded-full bg-green-500"></span>
        <span>{{ vms.filter(v => v.state === 'running').length }} Running</span>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading && vms.length === 0" class="p-12 text-center">
      <div class="animate-pulse text-[#adc6ff] text-2xl mb-4">⚡</div>
      <p class="text-[#8c909f] text-sm">Loading VMs...</p>
    </div>

    <!-- Empty State -->
    <div v-else-if="vms.length === 0" class="p-12 text-center">
      <span class="text-4xl mb-4 block">🖥️</span>
      <p class="text-[#8c909f]">No virtual machines configured</p>
      <p class="text-[#8c909f]/60 text-sm mt-2">Use the CLI to create your first VM</p>
    </div>

    <!-- VM List -->
    <div v-else class="divide-y divide-[#424753]/10">
      <div 
        v-for="vm in vms" 
        :key="vm.uuid"
        class="group p-4 hover:bg-[#1b2025] transition-all cursor-pointer"
        @click="emit('select', vm)"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <!-- Status Indicator -->
            <div class="relative">
              <div 
                class="w-3 h-3 rounded-full"
                :class="getStateColor(vm.state)"
              ></div>
              <div 
                v-if="vm.state === 'running'"
                class="absolute inset-0 w-3 h-3 rounded-full bg-green-500 animate-ping opacity-30"
              ></div>
            </div>
            
            <!-- VM Info -->
            <div>
              <h3 class="font-bold text-[#dee3e9] group-hover:text-[#adc6ff] transition-colors">
                {{ vm.name }}
              </h3>
              <div class="flex items-center gap-3 mt-1 text-xs text-[#8c909f] font-mono">
                <span>{{ vm.cpus }} CPU</span>
                <span>•</span>
                <span>{{ formatMemory(vm.memory_mb) }}</span>
                <span>•</span>
                <span>{{ vm.disk_gb }} GB</span>
              </div>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <button 
              v-if="vm.state === 'stopped'"
              @click.stop="emit('action', vm.name, 'start')"
              class="px-3 py-1.5 bg-green-500/20 text-green-400 rounded text-xs font-bold hover:bg-green-500/30 transition-colors"
            >
              START
            </button>
            <button 
              v-if="vm.state === 'running'"
              @click.stop="emit('action', vm.name, 'stop')"
              class="px-3 py-1.5 bg-yellow-500/20 text-yellow-400 rounded text-xs font-bold hover:bg-yellow-500/30 transition-colors"
            >
              STOP
            </button>
            <button 
              @click.stop="emit('action', vm.name, 'destroy')"
              class="px-3 py-1.5 bg-red-500/20 text-red-400 rounded text-xs font-bold hover:bg-red-500/30 transition-colors"
            >
              DESTROY
            </button>
          </div>
        </div>

        <!-- VM ID -->
        <div class="mt-2 ml-7">
          <span class="text-[10px] font-mono text-[#8c909f]/60">UUID: {{ vm.uuid.substring(0, 8) }}...</span>
        </div>
      </div>
    </div>
  </div>
</template>
