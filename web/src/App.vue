<script setup lang="ts">
import { ref, onMounted, onUnmounted, provide } from 'vue'
import Sidebar from './components/Sidebar.vue'
import TopNav from './components/TopNav.vue'
import VMList from './components/VMList.vue'
import VMDetail from './components/VMDetail.vue'
import SystemStats from './components/SystemStats.vue'
import ActionTiles from './components/ActionTiles.vue'
import ActivityLog from './components/ActivityLog.vue'
import LoginPage from './components/LoginPage.vue'
import type { VM, SystemInfo } from './types'

// Authentication state
const isAuthenticated = ref(false)
const apiKey = ref('')

// App data
const vms = ref<VM[]>([])
const selectedVM = ref<VM | null>(null)
const systemInfo = ref<SystemInfo | null>(null)
const loading = ref(true)
const error = ref('')
const refreshInterval = ref<number | null>(null)
const activeTab = ref('dashboard')

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:8080/api'

// Provide auth context to child components
provide('apiKey', apiKey)
provide('isAuthenticated', isAuthenticated)

function handleLogin(key: string) {
  apiKey.value = key
  isAuthenticated.value = true
  refreshData()
  refreshInterval.value = window.setInterval(refreshData, 5000)
}

function handleLogout() {
  apiKey.value = ''
  isAuthenticated.value = false
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
    refreshInterval.value = null
  }
  vms.value = []
  selectedVM.value = null
  systemInfo.value = null
}

async function fetchVMs() {
  try {
    const response = await fetch(`${API_BASE}/vms`, {
      headers: { 'Authorization': `Bearer ${apiKey.value}` }
    })
    if (!response.ok) {
      if (response.status === 401) {
        handleLogout()
        error.value = 'Session expired. Please login again.'
        return
      }
      throw new Error('Failed to fetch VMs')
    }
    vms.value = await response.json()
    error.value = ''
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  }
}

async function fetchSystemInfo() {
  try {
    const response = await fetch(`${API_BASE}/info`, {
      headers: { 'Authorization': `Bearer ${apiKey.value}` }
    })
    if (!response.ok) {
      if (response.status === 401) {
        handleLogout()
        return
      }
      throw new Error('Failed to fetch system info')
    }
    systemInfo.value = await response.json()
  } catch (err) {
    console.error('Error fetching system info:', err)
  }
}

async function refreshData() {
  if (!isAuthenticated.value) return
  await Promise.all([fetchVMs(), fetchSystemInfo()])
  loading.value = false
}

async function handleVMAction(vmName: string, action: 'start' | 'stop' | 'destroy') {
  try {
    const response = await fetch(`${API_BASE}/vms/${vmName}/${action}`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${apiKey.value}` }
    })
    if (!response.ok) {
      if (response.status === 401) {
        handleLogout()
        error.value = 'Session expired. Please login again.'
        return
      }
      throw new Error(`Failed to ${action} VM`)
    }
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

onUnmounted(() => {
  if (refreshInterval.value) clearInterval(refreshInterval.value)
})
</script>

<template>
  <LoginPage v-if="!isAuthenticated" @login="handleLogin" />
  
  <div v-else class="min-h-screen bg-[#0f1419] text-[#dee3e9] flex">
    <!-- Sidebar -->
    <Sidebar 
      :active-tab="activeTab" 
      @change-tab="activeTab = $event"
      :system-info="systemInfo"
    />
    
    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col md:ml-64">
      <!-- Top Navigation -->
      <TopNav 
        :system-info="systemInfo" 
        :loading="loading"
        @refresh="refreshData"
        @logout="handleLogout"
      />
      
      <!-- Main Content -->
      <main class="flex-1 p-6 md:p-10 pb-24 md:pb-10">
        <!-- Error Alert -->
        <div v-if="error" class="mb-6 p-4 bg-red-500/10 border border-red-500/30 rounded text-red-400 text-sm font-mono">
          {{ error }}
        </div>

        <!-- Hero Header -->
        <section class="mb-12">
          <div class="flex flex-col md:flex-row md:items-end justify-between gap-6">
            <div class="max-w-2xl">
              <span class="font-mono text-[10px] uppercase tracking-[0.3em] text-[#adc6ff] block mb-2">Aggregate Metrics</span>
              <h1 class="font-black text-5xl md:text-7xl tracking-tighter text-[#dee3e9] leading-none">
                Operational <span class="text-[#0969da]">Pulse</span>
              </h1>
            </div>
            <button class="bg-gradient-to-r from-[#adc6ff] to-[#0969da] text-[#002e68] px-8 py-3 rounded-sm font-bold text-sm tracking-widest flex items-center gap-2 active:scale-[0.98] transition-all uppercase">
              <span>+</span>
              New VM Instance
            </button>
          </div>
        </section>

        <!-- Stats Overview -->
        <SystemStats v-if="systemInfo" :info="systemInfo" class="mb-12" />

        <!-- Bento Grid Dashboard -->
        <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
          <!-- VM List -->
          <div class="md:col-span-8">
            <VMList 
              :vms="vms" 
              :loading="loading"
              @select="selectVM"
              @action="handleVMAction"
            />
          </div>
          
          <!-- Right Panel -->
          <div class="md:col-span-4 space-y-6">
            <!-- VM Detail or Host Stats -->
            <VMDetail 
              v-if="selectedVM" 
              :vm="selectedVM" 
              @close="closeDetail"
              @action="handleVMAction"
            />
            <div v-else class="bg-[#171c21] p-8 border border-[#424753]/10">
              <h3 class="font-bold text-xl text-[#dee3e9] mb-4">Quick Console</h3>
              <div class="grid grid-cols-2 gap-2">
                <button class="bg-[#30353a] p-3 font-mono text-[9px] uppercase hover:bg-[#0969da] hover:text-white transition-all rounded-sm">
                  Serial COM1
                </button>
                <button class="bg-[#30353a] p-3 font-mono text-[9px] uppercase hover:bg-[#0969da] hover:text-white transition-all rounded-sm">
                  VNC Display
                </button>
              </div>
            </div>
          </div>
          
          <!-- Activity Log -->
          <div class="md:col-span-12">
            <ActivityLog />
          </div>
        </div>

        <!-- Action Tiles -->
        <ActionTiles class="mt-12" />
      </main>
    </div>
  </div>
</template>
