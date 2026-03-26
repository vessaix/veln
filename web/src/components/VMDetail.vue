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
    case 'stopped': return 'text-[#8c909f]'
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
  <div class="bg-[#171c21] rounded-lg border border-[#424753]/20 overflow-hidden">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-[#424753]/20 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div 
          class="w-3 h-3 rounded-full"
          :class="vm.state === 'running' ? 'bg-green-500' : 'bg-[#8c909f]'"
        ></div>
        <h2 class="font-black tracking-tight">{{ vm.name }}</h2>
      </div>
      <button 
        @click="emit('close')"
        class="text-[#8c909f] hover:text-[#dee3e9] transition-colors"
      >
        ✕
      </button>
    </div>

    <div class="p-6 space-y-6">
      <!-- Status Badge -->
      <div class="inline-flex items-center gap-2 px-3 py-1.5 bg-[#1b2025] rounded">
        <span class="text-xs font-bold uppercase tracking-wider" :class="getStateColor(vm.state)">
          {{ vm.state }}
        </span>
      </div>

      <!-- Configuration -->
      <div class="space-y-4">
        <h3 class="text-xs font-bold uppercase tracking-widest text-[#8c909f]">Configuration</h3>
        
        <div class="grid grid-cols-2 gap-4">
          <div class="bg-[#1b2025] p-3 rounded">
            <div class="text-[10px] uppercase tracking-wider text-[#8c909f] mb-1">CPUs</div>
            <div class="font-mono text-lg text-[#adc6ff]">{{ vm.cpus }}</div>
          </div>
          <div class="bg-[#1b2025] p-3 rounded">
            <div class="text-[10px] uppercase tracking-wider text-[#8c909f] mb-1">Memory</div>
            <div class="font-mono text-lg text-[#adc6ff]">{{ formatMemory(vm.memory_mb) }}</div>
          </div>
          <div class="bg-[#1b2025] p-3 rounded">
            <div class="text-[10px] uppercase tracking-wider text-[#8c909f] mb-1">Disk</div>
            <div class="font-mono text-lg text-[#adc6ff]">{{ vm.disk_gb }} GB</div>
          </div>
          <div class="bg-[#1b2025] p-3 rounded">
            <div class="text-[10px] uppercase tracking-wider text-[#8c909f] mb-1">Network</div>
            <div class="font-mono text-sm text-[#adc6ff] uppercase">{{ vm.network_backend }}</div>
          </div>
        </div>
      </div>

      <!-- UUID -->
      <div class="space-y-2">
        <h3 class="text-xs font-bold uppercase tracking-widest text-[#8c909f]">Identifier</h3>
        <div class="bg-[#1b2025] p-3 rounded font-mono text-xs text-[#8c909f] break-all">
          {{ vm.uuid }}
        </div>
      </div>

      <!-- Console Info -->
      <div class="space-y-2">
        <h3 class="text-xs font-bold uppercase tracking-widest text-[#8c909f]">Console</h3>
        <div class="bg-[#1b2025] p-3 rounded flex items-center justify-between">
          <span class="text-sm text-[#dee3e9]">Type</span>
          <span class="font-mono text-sm text-[#adc6ff] uppercase">{{ vm.console_type }}</span>
        </div>
      </div>

      <!-- Actions -->
      <div class="pt-4 border-t border-[#424753]/20 space-y-3">
        <button 
          v-if="vm.state === 'stopped'"
          @click="emit('action', vm.name, 'start')"
          class="w-full py-3 bg-gradient-to-r from-green-600 to-green-500 text-white font-bold rounded hover:shadow-lg hover:shadow-green-500/20 transition-all"
        >
          START VM
        </button>
        <button 
          v-if="vm.state === 'running'"
          @click="emit('action', vm.name, 'stop')"
          class="w-full py-3 bg-gradient-to-r from-yellow-600 to-yellow-500 text-white font-bold rounded hover:shadow-lg hover:shadow-yellow-500/20 transition-all"
        >
          STOP VM
        </button>
        <button 
          @click="emit('action', vm.name, 'destroy')"
          class="w-full py-3 bg-[#1b2025] border border-red-500/30 text-red-400 font-bold rounded hover:bg-red-500/10 transition-all"
        >
          DESTROY VM
        </button>
      </div>
    </div>
  </div>
</template>
