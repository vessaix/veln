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
    case 'stopped': return 'bg-slate-500'
    case 'paused': return 'bg-blue-500'
    default: return 'bg-yellow-500'
  }
}

function getStateBorderColor(state: string): string {
  switch (state) {
    case 'running': return 'border-green-500'
    case 'stopped': return 'border-slate-500'
    case 'paused': return 'border-blue-500'
    default: return 'border-yellow-500'
  }
}

function formatMemory(mb: number): string {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)} GB`
  }
  return `${mb} MB`
}

const runningCount = (vms: VM[]) => vms.filter(v => v.state === 'running').length
const stoppedCount = (vms: VM[]) => vms.filter(v => v.state === 'stopped').length
</script>

<template>
  <div class="bg-[#171c21] p-8 border border-[#424753]/10">
    <div class="flex justify-between items-center mb-8">
      <h3 class="font-bold text-xl text-[#dee3e9]">Virtual Machines</h3>
      <div class="flex gap-4">
        <span class="font-mono text-[10px] text-[#adc6ff]">{{ runningCount(vms) }} RUNNING</span>
        <span class="font-mono text-[10px] text-[#8c909f]">{{ stoppedCount(vms) }} STOPPED</span>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading && vms.length === 0" class="p-12 text-center">
      <div class="animate-pulse text-[#adc6ff] text-2xl mb-4">⚡</div>
      <p class="text-[#8c909f] text-sm font-mono">Loading VMs...</p>
    </div>

    <!-- Empty State -->
    <div v-else-if="vms.length === 0" class="p-12 text-center">
      <span class="text-4xl mb-4 block">🖥️</span>
      <p class="text-[#8c909f]">No virtual machines configured</p>
      <p class="text-[#8c909f]/60 text-sm mt-2 font-mono">Use the CLI to create your first VM</p>
    </div>

    <!-- VM List -->
    <div v-else class="space-y-4">
      <div 
        v-for="vm in vms" 
        :key="vm.uuid"
        class="bg-[#1b2025] p-4 flex flex-col md:flex-row md:items-center justify-between gap-4 border-l-2 cursor-pointer hover:bg-[#252a30] transition-all"
        :class="getStateBorderColor(vm.state)"
        @click="emit('select', vm)"
      >
        <div class="flex items-center gap-4">
          <div class="w-10 h-10 bg-[#30353a] flex items-center justify-center rounded-sm">
            <span class="material-symbols-outlined text-[#8c909f]">terminal</span>
          </div>
          <div>
            <h4 class="text-sm font-bold text-[#dee3e9]">{{ vm.name }}</h4>
            <p class="font-mono text-[9px] text-[#8c909f] uppercase">
              {{ vm.state.toUpperCase() }} • {{ vm.cpus }} vCPUs • {{ formatMemory(vm.memory_mb) }} RAM
            </p>
          </div>
        </div>
        
        <div class="flex items-center gap-6">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full" :class="getStateColor(vm.state)"></span>
            <span class="font-mono text-[10px] uppercase" :class="{
              'text-green-500': vm.state === 'running',
              'text-slate-500': vm.state === 'stopped',
              'text-blue-500': vm.state === 'paused'
            }">{{ vm.state }}</span>
          </div>
          
          <div class="flex gap-2">
            <button 
              v-if="vm.state === 'stopped'"
              @click.stop="emit('action', vm.name, 'start')"
              class="w-8 h-8 flex items-center justify-center bg-[#30353a] hover:bg-green-500 transition-colors group rounded-sm"
            >
              <span class="material-symbols-outlined text-xs group-hover:text-white">play_arrow</span>
            </button>
            <button 
              v-if="vm.state === 'running'"
              @click.stop="emit('action', vm.name, 'stop')"
              class="w-8 h-8 flex items-center justify-center bg-[#30353a] hover:bg-red-500 transition-colors group rounded-sm"
            >
              <span class="material-symbols-outlined text-xs group-hover:text-white">stop</span>
            </button>
            <button class="w-8 h-8 flex items-center justify-center bg-[#30353a] hover:bg-[#0969da] transition-colors group rounded-sm">
              <span class="material-symbols-outlined text-xs group-hover:text-white">keyboard</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
