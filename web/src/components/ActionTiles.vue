<script setup lang="ts">
const actions = [
  { icon: 'auto_awesome_motion', label: 'Clone VM', color: 'hover:bg-[#0969da]' },
  { icon: 'camera', label: 'Snapshot', color: 'hover:bg-[#0969da]' },
  { icon: 'monitor_heart', label: 'Top / Iostat', color: 'hover:bg-[#0969da]' },
  { icon: 'terminal', label: 'Host Shell', color: 'hover:bg-[#0969da]' },
]
</script>

<template>
  <section>
    <h3 class="font-mono text-[10px] uppercase tracking-[0.3em] text-[#adc6ff] mb-6">Management Tools</h3>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div 
        v-for="action in actions" 
        :key="action.label"
        class="bg-[#1b2025] p-6 flex flex-col items-start gap-4 hover:bg-[#0969da] transition-all group cursor-pointer rounded-sm"
      >
        <span class="material-symbols-outlined text-[#adc6ff] group-hover:text-white text-2xl">{{ action.icon }}</span>
        <span class="font-mono text-[11px] uppercase font-bold text-[#dee3e9] group-hover:text-white">{{ action.label }}</span>
      </div>
    </div>
  </section>
</template>
