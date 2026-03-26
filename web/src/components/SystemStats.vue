<script setup lang="ts">
import type { SystemInfo } from '../types'

defineProps<{
  info: SystemInfo
}>()

function formatMemory(mb: number): string {
  if (mb >= 1024 * 1024) {
    return `${(mb / 1024 / 1024).toFixed(1)} TB`
  }
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)} GB`
  }
  return `${mb} MB`
}

function formatDisk(gb: number): string {
  if (gb >= 1024) {
    return `${(gb / 1024).toFixed(1)} TB`
  }
  return `${gb} GB`
}

// Mock usage percentages (in real app, would calculate from actual usage)
const cpuUsage = 75
const memoryUsage = 75.3
const diskUsage = 35
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- CPU Stat -->
    <div class="bg-[#171c21] p-6 border-b-2 border-[#adc6ff]">
      <p class="font-mono text-[10px] text-[#8c909f] uppercase tracking-widest mb-2">Total vCPUs Allocated</p>
      <div class="flex items-baseline gap-2">
        <span class="text-4xl font-black text-[#dee3e9]">{{ info.resources.cpu_cores }}</span>
        <span class="text-xs text-[#8c909f] font-mono">CORES</span>
      </div>
      <div class="w-full h-1 bg-[#30353a] mt-4">
        <div class="bg-[#adc6ff] h-full transition-all duration-500" :style="{ width: cpuUsage + '%' }"></div>
      </div>
    </div>

    <!-- Memory Stat -->
    <div class="bg-[#171c21] p-6 border-b-2 border-[#d0bcff]">
      <p class="font-mono text-[10px] text-[#8c909f] uppercase tracking-widest mb-2">Guest Memory Usage</p>
      <div class="flex items-baseline gap-2">
        <span class="text-4xl font-black text-[#dee3e9]">{{ formatMemory(info.resources.memory_mb) }}</span>
        <span class="text-xs text-[#8c909f] font-mono">TOTAL</span>
      </div>
      <div class="w-full h-1 bg-[#30353a] mt-4">
        <div class="bg-[#d0bcff] h-full transition-all duration-500" :style="{ width: memoryUsage + '%' }"></div>
      </div>
    </div>

    <!-- Disk Stat -->
    <div class="bg-[#171c21] p-6 border-b-2 border-[#0969da]">
      <p class="font-mono text-[10px] text-[#8c909f] uppercase tracking-widest mb-2">Total Storage (ZFS)</p>
      <div class="flex items-baseline gap-2">
        <span class="text-4xl font-black text-[#dee3e9]">{{ formatDisk(info.resources.disk_gb) }}</span>
        <span class="text-xs text-[#8c909f] font-mono">AVAILABLE</span>
      </div>
      <div class="w-full h-1 bg-[#30353a] mt-4">
        <div class="bg-[#0969da] h-full transition-all duration-500" :style="{ width: diskUsage + '%' }"></div>
      </div>
    </div>
  </div>
</template>
