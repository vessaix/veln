<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  login: [apiKey: string]
}>()

const apiKey = ref('')
const error = ref('')
const loading = ref(false)

async function handleSubmit() {
  if (!apiKey.value.trim()) {
    error.value = 'Please enter an API key'
    return
  }

  error.value = ''
  loading.value = true

  try {
    const response = await fetch(`${import.meta.env.VITE_API_URL}/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ api_key: apiKey.value }),
    })

    const data = await response.json()

    if (data.success) {
      emit('login', apiKey.value)
    } else {
      error.value = data.error || 'Invalid API key'
    }
  } catch (err) {
    error.value = 'Failed to connect to API server'
    console.error('Login error:', err)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen bg-[#0f1419] text-[#dee3e9] flex flex-col relative overflow-hidden">
    <!-- Background Accents -->
    <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-[#0969da]/5 rounded-full blur-[120px] pointer-events-none"></div>
    
    <!-- Header -->
    <header class="w-full top-0 sticky bg-[#0f1419] z-50 border-b border-[#424753]/20">
      <div class="flex items-center justify-between px-6 py-4 w-full max-w-screen-2xl mx-auto">
        <div class="flex items-center gap-2">
          <span class="text-[#adc6ff] text-2xl">⚡</span>
          <span class="font-black tracking-tighter uppercase text-xl text-[#adc6ff]">VELN</span>
        </div>
        <div class="font-mono text-[10px] tracking-[0.2em] text-[#8c909f]">SYSTEM_AUTH_REQUIRED</div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-grow flex items-center justify-center p-6 relative">
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-12 max-w-6xl w-full items-center relative z-10">
        <!-- Hero Branding Column -->
        <div class="lg:col-span-7 space-y-8">
          <div class="space-y-2">
            <span class="font-mono text-[#adc6ff] text-xs tracking-widest uppercase">bhyve VM management platform</span>
            <h1 class="font-black text-5xl md:text-7xl tracking-tighter text-[#dee3e9] leading-none">
              VELN <span class="text-[#adc6ff]">CARBON</span>
            </h1>
          </div>
          <p class="text-[#8c909f] max-w-md text-lg leading-relaxed">
            Access high-performance virtualization nodes. Precision-engineered for technical speed and cryptographic security.
          </p>
          <div class="flex gap-4 pt-4">
            <div class="bg-[#171c21] px-4 py-3 rounded border-l-2 border-[#adc6ff]">
              <div class="font-mono text-[10px] text-[#8c909f] uppercase">Node Status</div>
              <div class="font-mono text-sm text-[#dee3e9]">API_AUTHENTICATION</div>
            </div>
            <div class="bg-[#171c21] px-4 py-3 rounded border-l-2 border-[#d0bcff]">
              <div class="font-mono text-[10px] text-[#8c909f] uppercase">Version</div>
              <div class="font-mono text-sm text-[#dee3e9]">v0.1.0</div>
            </div>
          </div>
        </div>

        <!-- Login Interface Column -->
        <div class="lg:col-span-5">
          <div class="bg-[rgba(23,28,33,0.7)] backdrop-blur-[20px] border border-[#424753]/15 p-8 md:p-10 rounded-lg shadow-2xl relative">
            <!-- Subtle Corner Accent -->
            <div class="absolute top-0 right-0 w-12 h-12 border-t border-r border-[#adc6ff]/30 rounded-tr-lg"></div>
            
            <form @submit.prevent="handleSubmit" class="space-y-6">
              <!-- API Key Input -->
              <div class="space-y-1">
                <label class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f] px-1" for="api_key">
                  API Access Key
                </label>
                <div class="relative group">
                  <input 
                    id="api_key"
                    v-model="apiKey"
                    type="password"
                    placeholder="veln-api-key-xxx"
                    class="w-full bg-[#171c21] border-none focus:ring-0 text-[#dee3e9] font-mono text-sm py-3 px-4 transition-all outline-none"
                    :disabled="loading"
                  />
                  <div class="absolute bottom-0 left-0 w-full h-[2px] bg-[#adc6ff] shadow-[0_2px_10px_rgba(173,198,255,0.4)] opacity-0 group-focus-within:opacity-100 transition-opacity"></div>
                </div>
              </div>

              <!-- Error Message -->
              <div v-if="error" class="p-3 bg-red-500/10 border border-red-500/30 rounded text-red-400 text-sm font-mono">
                {{ error }}
              </div>

              <!-- Submit Button -->
              <button 
                type="submit"
                :disabled="loading"
                class="w-full mt-8 bg-gradient-to-r from-[#adc6ff] to-[#0969da] text-[#002e68] font-black py-4 px-6 rounded shadow-[0_0_20px_rgba(173,198,255,0.2)] hover:shadow-[0_0_30px_rgba(173,198,255,0.4)] active:scale-[0.98] transition-all flex items-center justify-center gap-3 group disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <span v-if="loading" class="animate-spin">⟳</span>
                <span v-else>INIT_SESSION</span>
                <span v-if="!loading" class="text-xl group-hover:translate-x-1 transition-transform">⚡</span>
              </button>
            </form>

            <div class="mt-8 flex justify-center">
              <span class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f]">
                Enter your API key to continue
              </span>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="w-full border-t border-[#424753]/15 bg-[#0f1419]">
      <div class="flex flex-col md:flex-row justify-between items-center px-8 py-6 w-full max-w-screen-2xl mx-auto">
        <div class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f]">
          © 2024 VESSAIX. ALL RIGHTS RESERVED.
        </div>
        <div class="flex gap-8 mt-4 md:mt-0">
          <a href="https://github.com/vessaix/veln" class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f] hover:text-[#adc6ff] transition-all">
            GITHUB
          </a>
          <a href="#" class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f] hover:text-[#adc6ff] transition-all">
            DOCUMENTATION
          </a>
        </div>
      </div>
    </footer>

    <!-- Decorative Screen Texture -->
    <div class="fixed inset-0 pointer-events-none opacity-[0.03] bg-[url('https://grainy-gradients.vercel.app/noise.svg')] mix-blend-overlay"></div>
  </div>
</template>
