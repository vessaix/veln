<script setup lang="ts">
import { ref, onMounted } from 'vue'

const emit = defineEmits<{
  login: [apiKey: string]
}>()

const apiKey = ref('')
const error = ref('')
const loading = ref(false)

// Dynamic copyright year
const currentYear = new Date().getFullYear()

onMounted(() => {
  // Check if already logged in
  const existingKey = localStorage.getItem('veln_api_key')
  if (existingKey) {
    apiKey.value = existingKey
  }
})

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
      // Store API key for persistence
      localStorage.setItem('veln_api_key', apiKey.value)
      localStorage.setItem('veln_user', JSON.stringify(data))
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
    <!-- Background Accents - Tonal layering -->
    <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] md:w-[800px] md:h-[800px] bg-[#0969da]/5 rounded-full blur-[120px] pointer-events-none"></div>
    <div class="absolute top-1/4 right-1/4 w-[400px] h-[400px] bg-[#adc6ff]/3 rounded-full blur-[100px] pointer-events-none"></div>
    
    <!-- Header -->
    <header class="w-full sticky top-0 z-50 bg-[#0f1419]">
      <div class="flex items-center justify-between px-6 py-4 w-full max-w-screen-2xl mx-auto">
        <div class="flex items-center gap-3">
          <span class="material-symbols-outlined text-[#adc6ff] text-2xl" style="font-variation-settings: 'FILL' 1;">bolt</span>
          <span class="font-black tracking-tighter uppercase text-xl text-[#adc6ff]">VELN</span>
        </div>
        <div class="font-mono text-[10px] tracking-[0.2em] text-[#8c909f] uppercase">System Authentication Required</div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-grow flex items-center justify-center p-6 relative">
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-12 max-w-6xl w-full items-center relative z-10">
        
        <!-- Hero Branding Column -->
        <div class="lg:col-span-7 space-y-8">
          <div class="space-y-2">
            <span class="font-mono text-[#adc6ff] text-xs tracking-[0.3em] uppercase block mb-2">bhyve virtualization platform</span>
            <h1 class="font-black text-5xl md:text-7xl tracking-tighter text-[#dee3e9] leading-none">
              VELN <span class="text-[#0969da]">TERMINAL</span>
            </h1>
          </div>
          <p class="text-[#8c909f] max-w-md text-lg leading-relaxed">
            High-performance virtualization management for FreeBSD. Secure, fast, and engineered for technical precision.
          </p>
          <div class="flex flex-wrap gap-4 pt-4">
            <div class="bg-[#171c21] px-5 py-4 rounded-sm border-l-2 border-[#adc6ff] min-w-[140px]">
              <div class="font-mono text-[10px] text-[#8c909f] uppercase tracking-widest mb-1">API Status</div>
              <div class="font-mono text-sm text-[#dee3e9] flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
                ONLINE
              </div>
            </div>
            <div class="bg-[#171c21] px-5 py-4 rounded-sm border-l-2 border-[#d0bcff] min-w-[140px]">
              <div class="font-mono text-[10px] text-[#8c909f] uppercase tracking-widest mb-1">Version</div>
              <div class="font-mono text-sm text-[#dee3e9]">v0.1.0</div>
            </div>
          </div>
        </div>

        <!-- Login Interface Column -->
        <div class="lg:col-span-5">
          <!-- Glass morphism panel -->
          <div class="bg-[rgba(27,32,37,0.6)] backdrop-blur-[20px] border border-[#424753]/15 p-8 md:p-10 rounded-lg shadow-[0_8px_32px_rgba(173,198,255,0.06)] relative">
            <!-- Subtle Corner Accent -->
            <div class="absolute top-0 right-0 w-12 h-12 border-t border-r border-[#adc6ff]/20 rounded-tr-lg"></div>
            
            <form @submit.prevent="handleSubmit" class="space-y-6">
              <!-- API Key Input -->
              <div class="space-y-2">
                <label class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f] block" for="api_key">
                  API Access Key
                </label>
                <div class="relative">
                  <input 
                    id="api_key"
                    v-model="apiKey"
                    type="password"
                    placeholder="veln-api-key-xxx..."
                    required
                    autocomplete="off"
                    class="w-full bg-[#171c21] text-[#dee3e9] font-mono text-sm py-3.5 px-4 rounded-sm placeholder-[#8c909f]/50 border-none border-b-2 border-transparent focus:border-b-[#adc6ff] focus:outline-none focus:shadow-[0_2px_20px_rgba(173,198,255,0.3)] transition-all"
                    :disabled="loading"
                  />
                </div>
                <p class="font-mono text-[9px] text-[#8c909f]/70 mt-1">Enter your authentication token</p>
              </div>

              <!-- Error Message -->
              <div v-if="error" class="p-3 bg-red-500/10 border border-red-500/30 rounded-sm">
                <p class="font-mono text-[11px] text-red-400">{{ error }}</p>
              </div>

              <!-- Submit Button - Gradient with kinetic energy -->
              <button 
                type="submit"
                :disabled="loading"
                class="w-full bg-gradient-to-r from-[#adc6ff] to-[#0969da] text-[#002e68] font-black py-4 px-6 rounded-sm shadow-[0_0_20px_rgba(173,198,255,0.15)] hover:shadow-[0_0_30px_rgba(173,198,255,0.25)] active:scale-[0.98] transition-all flex items-center justify-center gap-3 group disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <span v-if="loading" class="font-mono text-sm">AUTHENTICATING...</span>
                <span v-else class="font-mono text-sm">AUTHENTICATE</span>
                <span v-if="!loading" class="material-symbols-outlined text-xl group-hover:translate-x-1 transition-transform">arrow_forward</span>
                <span v-else class="material-symbols-outlined text-xl animate-spin">sync</span>
              </button>
            </form>

            <!-- Help Link -->
            <div class="mt-8 pt-6 border-t border-[#424753]/10 flex justify-center">
              <a href="https://github.com/vessaix/veln" class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f] hover:text-[#adc6ff] transition-colors">
                Documentation →
              </a>
            </div>
          </div>
          
          <!-- Quick Tip - Terminal chip style -->
          <div class="mt-4 bg-[#171c21]/50 px-4 py-3 rounded-sm">
            <p class="font-mono text-[9px] text-[#8c909f]">
              <span class="text-[#adc6ff]">TIP:</span> Generate keys with <code class="text-[#d0bcff]">veln api-key generate</code>
            </p>
          </div>
        </div>
      </div>
    </main>

    <!-- Footer - Dynamic copyright year -->
    <footer class="w-full border-t border-[#424753]/10 bg-[#0f1419]">
      <div class="flex flex-col md:flex-row justify-between items-center px-8 py-6 w-full max-w-screen-2xl mx-auto">
        <div class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f]/60">
          © {{ currentYear }} VESSAIX. All rights reserved.
        </div>
        <div class="flex gap-8 mt-4 md:mt-0">
          <a href="https://github.com/vessaix/veln" class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f]/60 hover:text-[#adc6ff] transition-colors">GitHub</a>
          <a href="#" class="font-mono text-[10px] uppercase tracking-widest text-[#8c909f]/60 hover:text-[#adc6ff] transition-colors">Documentation</a>
        </div>
      </div>
    </footer>

    <!-- Decorative Screen Texture -->
    <div class="fixed inset-0 pointer-events-none opacity-[0.02] bg-[url('data:image/svg+xml,%3Csvg viewBox=%220 0 200 200%22 xmlns=%22http://www.w3.org/2000/svg%22%3E%3Cfilter id=%22noise%22%3E%3CfeTurbulence type=%22fractalNoise%22 baseFrequency=%220.65%22 numOctaves=%223%22 stitchTiles=%22stitch%22/%3E%3C/filter%3E%3Crect width=%22100%25%22 height=%22100%25%22 filter=%22url(%23noise)%22/%3E%3C/svg%3E')] mix-blend-overlay"></div>
  </div>
</template>
