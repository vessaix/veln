<script setup lang="ts">
import type { SystemInfo } from '../types'

defineProps<{
  systemInfo: SystemInfo | null
  loading: boolean
}>()

defineEmits<{
  refresh: []
  logout: []
}>()
</script>

<template>
  <header class="flex items-center justify-between px-6 py-4 w-full sticky top-0 z-50 bg-[#0f1419]/80 backdrop-blur-xl border-b border-[#424753]/20">
    <div class="flex items-center gap-4">
      <span class="material-symbols-outlined text-[#0969da]" style="font-variation-settings: 'FILL' 1;">terminal</span>
      <span class="font-black tracking-tighter text-[#0969da] text-xl">VELN</span>
    </div>
    
    <div class="flex items-center gap-6">
      <div class="hidden md:flex gap-8 items-center">
        <span class="font-bold text-[#adc6ff]">Hypervisor</span>
        <span class="font-mono text-[11px] uppercase tracking-widest text-slate-500 hover:text-slate-300 cursor-pointer transition-colors">Virtual Machines</span>
        <span class="font-mono text-[11px] uppercase tracking-widest text-slate-500 hover:text-slate-300 cursor-pointer transition-colors">Storage Pools</span>
      </div>
      
      <div class="flex items-center gap-4">
        <button 
          @click="$emit('refresh')"
          class="p-2 rounded hover:bg-[#1b2025] transition-colors"
          :class="{ 'animate-spin': loading }"
        >
          <span class="text-[#adc6ff]">↻</span>
        </button>
        
        <button 
          @click="$emit('logout')"
          class="px-3 py-1.5 text-xs font-bold text-[#8c909f] hover:text-red-400 transition-colors font-mono uppercase"
        >
          Logout
        </button>
        
        <div class="w-8 h-8 rounded-full bg-[#30353a] border border-[#424753] flex items-center justify-center text-[10px] font-bold overflow-hidden">
          <span class="material-symbols-outlined text-[#8c909f]">person</span>
        </div>
      </div>
    </div>
  </header>
</template>
