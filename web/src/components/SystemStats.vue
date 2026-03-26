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
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <!-- Memory Stat -->
    <div class="bg-[#171c21] p-4 rounded-lg border border-[#424753]/20 relative overflow-hidden">
      <div class="absolute top-0 right-0 w-24 h-24 bg-[#adc6ff]/5 rounded-full blur-2xl"></div>
      <div class="relative">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-[#adc6ff]">🧠</span>
          <span class="text-[10px] uppercase tracking-widest text-[#8c909f]">Total Memory</span>
        </div>
        <div class="font-mono text-2xl font-bold text-[#dee3e9]">
          {{ formatMemory(info.resources.memory_mb) }}
        </div>
      </div>
    </div>

    <!-- CPU Stat -->
    <div class="bg-[#171c21] p-4 rounded-lg border border-[#424753]/20 relative overflow-hidden">
      <div class="absolute top-0 right-0 w-24 h-24 bg-[#adc6ff]/5 rounded-full blur-2xl"></div>
      <div class="relative">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-[#adc6ff]">⚡</span>
          <span class="text-[10px] uppercase tracking-widest text-[#8c909f]">CPU Cores</span>
        </div>
        <div class="font-mono text-2xl font-bold text-[#dee3e9]">
          {{ info.resources.cpu_cores }}
        </div>
      </div>
    </div>

    <!-- Disk Stat -->
    <div class="bg-[#171c21] p-4 rounded-lg border border-[#424753]/20 relative overflow-hidden">
      <div class="absolute top-0 right-0 w-24 h-24 bg-[#adc6ff]/5 rounded-full blur-2xl"></div>
      <div class="relative">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-[#adc6ff]">💾</span>
          <span class="text-[10px] uppercase tracking-widest text-[#8c909f]">Storage Pool</span>
        </div>
        <div class="font-mono text-2xl font-bold text-[#dee3e9]">
          {{ formatDisk(info.resources.disk_gb) }}
        </div>
        <div class="text-xs text-[#8c909f] mt-1">{{ info.pool }}</div>
      </div>
    </div>
  </div>
</template>
