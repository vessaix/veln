<script setup lang="ts">
import type { VM } from '../types'

defineProps<{
  vm: VM
}>()

const emit = defineEmits<{
  close: []
  action: [vmName: string, action: 'start' | 'stop' | 'destroy']
}>()

function getStateColor(state: string): string {
  switch (state) {
    case 'running': return 'text-green-400'
    case 'stopped': return 'text-slate-400'
    case 'paused': return 'text-blue-400'
    default: return 'text-yellow-400'
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
  <div class="bg-[#171c21] p-8 border-l-4 border-[#0969da]">
    <div class="mb-8">
      <span class="material-symbols-outlined text-[#adc6ff] mb-4 text-3xl">dns</span>
      <h3 class="font-bold text-xl text-[#dee3e9]">{{ vm.name }}</h3>
      <p class="font-mono text-[10px] text-[#8c909f] uppercase tracking-wider">{{ vm.uuid.substring(0, 8) }}...</p>
    </div>
    
    <div class="space-y-6">
      <!-- Status Badge -->
      <div class="inline-flex items-center gap-2 px-3 py-1.5 bg-[#1b2025] rounded-sm">
        <span class="text-xs font-bold uppercase tracking-wider" :class="getStateColor(vm.state)">
          {{ vm.state }}
        </span>
      </div>

      <!-- Configuration Grid -->
      <div class="space-y-4">
        <div>
          <div class="flex justify-between font-mono text-[10px] mb-2">
            <span class="text-[#8c909f]">CPUs</span>
            <span class="text-[#adc6ff]">{{ vm.cpus }} vCPUs</span>
          </div>
          <div class="w-full h-1 bg-[#30353a]">
            <div class="bg-[#adc6ff] h-full" :style="{ width: (vm.cpus / 16 * 100) + '%' }"></div>
          </div>
        </div>
        
        <div>
          <div class="flex justify-between font-mono text-[10px] mb-2">
            <span class="text-[#8c909f]">Memory</span>
            <span class="text-[#adc6ff]">{{ formatMemory(vm.memory_mb) }}</span>
          </div>
          <div class="w-full h-1 bg-[#30353a]">
            <div class="bg-[#adc6ff] h-full" :style="{ width: (vm.memory_mb / 32768 * 100) + '%' }"></div>
          </div>
        </div>
        
        <div>
          <div class="flex justify-between font-mono text-[10px] mb-2">
            <span class="text-[#8c909f]">Disk</span>
            <span class="text-[#adc6ff]">{{ vm.disk_gb }} GB</span>
          </div>
          <div class="w-full h-1 bg-[#30353a]">
            <div class="bg-[#adc6ff] h-full" :style="{ width: (vm.disk_gb / 100 * 100) + '%' }"></div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="pt-6 border-t border-[#424753]/20 space-y-3">
        <button 
          v-if="vm.state === 'stopped'"
          @click="emit('action', vm.name, 'start')"
          class="w-full py-3 bg-gradient-to-r from-green-600 to-green-500 text-white font-bold rounded-sm hover:shadow-lg hover:shadow-green-500/20 transition-all"
        >
          START VM
        </button>
        <button 
          v-if="vm.state === 'running'"
          @click="emit('action', vm.name, 'stop')"
          class="w-full py-3 bg-gradient-to-r from-yellow-600 to-yellow-500 text-white font-bold rounded-sm hover:shadow-lg hover:shadow-yellow-500/20 transition-all"
        >
          STOP VM
        </button>
        <button 
          @click="emit('action', vm.name, 'destroy')"
          class="w-full py-3 bg-[#1b2025] border border-red-500/30 text-red-400 font-bold rounded-sm hover:bg-red-500/10 transition-all"
        >
          DESTROY VM
        </button>
        <button 
          @click="emit('close')"
          class="w-full py-2 text-xs text-[#8c909f] hover:text-[#dee3e9] transition-colors"
        >
          Close Details
        </button>
      </div>
    </div>
  </div>
</template>
