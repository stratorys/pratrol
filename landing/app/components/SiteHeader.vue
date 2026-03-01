<template>
  <header class="border-b border-sky-100 bg-white/95">
    <div class="shell flex items-center justify-between py-4">
      <NuxtLink to="/" class="text-xl font-semibold tracking-tight text-slate-950">Pratrol</NuxtLink>
      <nav class="hidden gap-7 text-sm font-semibold text-slate-700 md:flex">
        <NuxtLink to="/" :class="active === 'home' ? 'text-blue-700' : 'hover:text-blue-700'">Home</NuxtLink>
        <NuxtLink to="/scoring" :class="active === 'scoring' ? 'text-blue-700' : 'hover:text-blue-700'">Scoring</NuxtLink>
        <NuxtLink to="/blog" :class="active === 'blog' ? 'text-blue-700' : 'hover:text-blue-700'">Blog</NuxtLink>
      </nav>
      <a href="#" class="btn-primary">Install on GitHub</a>
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  active: 'home' | 'scoring' | 'blog'
}>()
</script>
