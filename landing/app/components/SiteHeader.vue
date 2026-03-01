<template>
  <header class="border-b border-slate-100 bg-white/95 backdrop-blur-md sticky top-0 z-50">
    <div class="shell flex items-center justify-between py-4">
      <NuxtLink to="/" class="flex items-center gap-3 group">
        <img src="/logo.png" alt="Pratrol Logo" class="h-9 w-auto" />
        <span class="text-xl font-bold tracking-tight text-slate-900">Pratrol</span>
      </NuxtLink>
      <nav class="hidden gap-8 text-[13px] font-bold uppercase tracking-wider text-slate-500 md:flex">
        <NuxtLink to="/" :class="active === 'home' ? 'text-slate-900' : 'hover:text-slate-900'">Home</NuxtLink>
        <NuxtLink to="/scoring" :class="active === 'scoring' ? 'text-slate-900' : 'hover:text-slate-900'">Scoring</NuxtLink>
        <NuxtLink to="/blog" :class="active === 'blog' ? 'text-slate-900' : 'hover:text-slate-900'">Blog</NuxtLink>
      </nav>
      <div class="flex items-center gap-4">
        <a href="#" class="btn-primary">Install on GitHub</a>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  active: 'home' | 'scoring' | 'blog'
}>()
</script>
