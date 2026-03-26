<script setup lang="ts">
const logs = [
  { time: '16:45:01', tag: '[BHYVE]', tagColor: 'text-[#adc6ff]', message: "VM 'web-frontend-01' started on tap0." },
  { time: '16:42:15', tag: '[ZFS]', tagColor: 'text-[#d0bcff]', message: "Snapshot created: tank/vms/win-ads-primary@daily_backup" },
  { time: '16:30:10', tag: '[INFO]', tagColor: 'text-[#adc6ff]', message: "PCI passthrough initialized for device 0:2:0." },
  { time: '16:15:33', tag: '[BHYVE]', tagColor: 'text-[#adc6ff]', message: "VM 'db-prod-cluster-b' stopped gracefully." },
  { time: '15:58:22', tag: '[ZFS]', tagColor: 'text-[#d0bcff]', message: "Dataset zroot/veln/vms created successfully." },
]
</script>

<template>
  <div class="bg-[#0a0f13] p-8 border border-[#424753]/10">
    <div class="flex justify-between items-center mb-8">
      <h3 class="font-bold text-xl text-[#dee3e9]">Hypervisor Logs</h3>
      <span class="font-mono text-[10px] text-[#8c909f]">FILTER: /VAR/LOG/VELN.LOG</span>
    </div>
    <div class="space-y-3 font-mono text-[11px] h-32 overflow-y-auto no-scrollbar">
      <div 
        v-for="(log, index) in logs" 
        :key="index"
        class="flex gap-4 opacity-80 hover:opacity-100 transition-opacity"
      >
        <span class="text-[#8c909f]">{{ log.time }}</span>
        <span :class="log.tagColor">{{ log.tag }}</span>
        <span class="text-[#dee3e9]">{{ log.message }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
</style>
