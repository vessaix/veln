<script setup lang="ts">
import type { SystemInfo } from '../types'

defineProps<{
  activeTab: string
  systemInfo: SystemInfo | null
}>()

defineEmits<{
  changeTab: [tab: string]
}>()
</script>

<template>
  <aside class="fixed left-0 top-0 h-full flex flex-col pt-20 bg-[#0f1419] w-64 border-r-0 z-40 hidden md:flex">
    <!-- Server Info -->
    <div class="px-6 mb-8">
      <h2 class="font-mono text-[11px] uppercase tracking-widest text-[#0969da] opacity-70">BHYVE_SERVER_01</h2>
    </div>
    
    <!-- Navigation -->
    <nav class="flex flex-col">
      <div 
        :class="activeTab === 'dashboard' ? 'text-[#adc6ff] bg-[#1b2025] border-r-2 border-[#0969da]' : 'text-slate-500 hover:text-slate-300 hover:bg-[#171c21]'"
        class="py-3 px-6 flex items-center gap-3 cursor-pointer transition-all duration-150"
        @click="$emit('changeTab', 'dashboard')"
      >
        <span class="material-symbols-outlined">grid_view</span>
        <span class="font-mono text-[11px] uppercase tracking-widest">Dashboard</span>
      </div>
      
      <div 
        :class="activeTab === 'vms' ? 'text-[#adc6ff] bg-[#1b2025] border-r-2 border-[#0969da]' : 'text-slate-500 hover:text-slate-300 hover:bg-[#171c21]'"
        class="py-3 px-6 flex items-center gap-3 cursor-pointer transition-all duration-150"
        @click="$emit('changeTab', 'vms')"
      >
        <span class="material-symbols-outlined">computer</span>
        <span class="font-mono text-[11px] uppercase tracking-widest">Virtual Machines</span>
      </div>
      
      <div 
        :class="activeTab === 'storage' ? 'text-[#adc6ff] bg-[#1b2025] border-r-2 border-[#0969da]' : 'text-slate-500 hover:text-slate-300 hover:bg-[#171c21]'"
        class="py-3 px-6 flex items-center gap-3 cursor-pointer transition-all duration-150"
        @click="$emit('changeTab', 'storage')"
      >
        <span class="material-symbols-outlined">database</span>
        <span class="font-mono text-[11px] uppercase tracking-widest">Storage & ZFS</span>
      </div>
      
      <div 
        :class="activeTab === 'network' ? 'text-[#adc6ff] bg-[#1b2025] border-r-2 border-[#0969da]' : 'text-slate-500 hover:text-slate-300 hover:bg-[#171c21]'"
        class="py-3 px-6 flex items-center gap-3 cursor-pointer transition-all duration-150"
        @click="$emit('changeTab', 'network')"
      >
        <span class="material-symbols-outlined">lan</span>
        <span class="font-mono text-[11px] uppercase tracking-widest">Networking</span>
      </div>
      
      <div 
        :class="activeTab === 'config' ? 'text-[#adc6ff] bg-[#1b2025] border-r-2 border-[#0969da]' : 'text-slate-500 hover:text-slate-300 hover:bg-[#171c21]'"
        class="py-3 px-6 flex items-center gap-3 cursor-pointer transition-all duration-150"
        @click="$emit('changeTab', 'config')"
      >
        <span class="material-symbols-outlined">settings</span>
        <span class="font-mono text-[11px] uppercase tracking-widest">Hypervisor Config</span>
      </div>
    </nav>
    
    <!-- Host Status Panel -->
    <div class="mt-auto p-6">
      <div class="bg-[#171c21] p-4 rounded-sm border-l-2 border-[#adc6ff]">
        <p class="text-[10px] font-mono text-[#adc6ff] mb-1 uppercase tracking-widest">FreeBSD Host Status</p>
        <div class="space-y-2 mt-2">
          <div class="flex justify-between items-center text-[10px] font-mono">
            <span class="text-[#8c909f]">UPTIME</span>
            <span class="text-[#dee3e9]">--</span>
          </div>
          <div class="flex justify-between items-center text-[10px] font-mono">
            <span class="text-[#8c909f]">LOAD</span>
            <span class="text-[#dee3e9]">--</span>
          </div>
          <div class="flex justify-between items-center text-[10px] font-mono">
            <span class="text-[#8c909f]">ZFS POOL</span>
            <span class="text-green-400">ONLINE</span>
          </div>
          <div class="flex justify-between items-center text-[10px] font-mono">
            <span class="text-[#8c909f]">POOL</span>
            <span class="text-[#dee3e9]">{{ systemInfo?.pool || 'zroot' }}</span>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>
