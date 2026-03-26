<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import VMList from './components/VMList.vue'
import VMDetail from './components/VMDetail.vue'
import SystemStats from './components/SystemStats.vue'
import type { VM, SystemInfo } from './types'

const vms = ref<VM[]>([])
const selectedVM = ref<VM | null>(null)
const systemInfo = ref<SystemInfo | null>(null)
const loading = ref(true)
const error = ref('')
const refreshInterval = ref<number | null>(null)

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:8080/api'

async function fetchVMs() {
  try {
    const response = await fetch(`${API_BASE}/vms`)
    if (!response.ok) throw new Error('Failed to fetch VMs')
    vms.value = await response.json()
    error.value = ''
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
    console.error('Error fetching VMs:', err)
  }
}

async function fetchSystemInfo() {
  try {
    const response = await fetch(`${API_BASE}/info`)
    if (!response.ok) throw new Error('Failed to fetch system info')
    systemInfo.value = await response.json()
  } catch (err) {
    console.error('Error fetching system info:', err)
  }
}

async function refreshData() {
  await Promise.all([fetchVMs(), fetchSystemInfo()])
  loading.value = false
}

async function handleVMAction(vmName: string, action: 'start' | 'stop' | 'destroy') {
  try {
    const response = await fetch(`${API_BASE}/vms/${vmName}/${action}`, {
      method: 'POST'
    })
    if (!response.ok) throw new Error(`Failed to ${action} VM`)
    await refreshData()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Action failed'
  }
}

function selectVM(vm: VM) {
  selectedVM.value = vm
}

function closeDetail() {
  selectedVM.value = null
}

onMounted(() => {
  refreshData()
  refreshInterval.value = window.setInterval(refreshData, 5000)
})

onUnmounted(() => {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
  }
})
</script>

<template>
  <div class="min-h-screen bg-[#0f1419] text-[#dee3e9] font-sans">
    <!-- Top Navigation -->
    <header class="sticky top-0 z-50 bg-[#0f1419]/95 backdrop-blur-xl border-b border-[#424753]/20">
      <div class="flex items-center justify-between px-6 py-4 max-w-screen-2xl mx-auto">
        <div class="flex items-center gap-3">
          <span class="text-[#adc6ff] text-2xl">⚡</span>
          <span class="font-black tracking-tighter uppercase text-xl text-[#adc6ff]">VELN</span>
          <span class="text-[10px] uppercase tracking-widest text-[#8c909f] ml-2">Web Console</span>
        </div>
        <div class="flex items-center gap-4">
          <div v-if="systemInfo" class="hidden md:flex items-center gap-4 text-xs font-mono">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
              <span class="text-[#8c909f]">{{ systemInfo.pool }}</span>
            </div>
          </div>
          <button 
            @click="refreshData"
            class="p-2 rounded hover:bg-[#1b2025] transition-colors"
            :class="{ 'animate-spin': loading }"
          >
            <span class="text-[#adc6ff]">↻</span>
          </button>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="p-6 max-w-screen-2xl mx-auto">
      <!-- Error Alert -->
      <div v-if="error" class="mb-6 p-4 bg-red-500/10 border border-red-500/30 rounded text-red-400 text-sm">
        {{ error }}
      </div>

      <!-- System Stats -->
      <SystemStats v-if="systemInfo" :info="systemInfo" class="mb-6" />

      <!-- VM List -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div class="lg:col-span-2">
          <VMList 
            :vms="vms" 
            :loading="loading"
            @select="selectVM"
            @action="handleVMAction"
          />
        </div>
        
        <!-- VM Detail Panel -->
        <div class="lg:col-span-1">
          <VMDetail 
            v-if="selectedVM" 
            :vm="selectedVM" 
            @close="closeDetail"
            @action="handleVMAction"
          />
          <div v-else class="bg-[#171c21] rounded-lg p-8 text-center border border-[#424753]/20">
            <span class="text-4xl mb-4 block">📊</span>
            <p class="text-[#8c909f] text-sm">Select a VM to view details</p>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>
