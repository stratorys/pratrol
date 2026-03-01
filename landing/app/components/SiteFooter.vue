<template>
  <footer class="mt-32 border-t border-slate-100 bg-white py-12">
    <div class="shell flex flex-col items-center justify-between gap-6 md:flex-row">
      <div class="flex items-center gap-4">
        <img src="/logo.png" alt="Pratrol Logo" class="h-6 w-auto grayscale opacity-50" />
        <p class="text-[13px] text-slate-500 font-medium">
          Build with ❤️ during <span class="text-slate-900">Mistral Worldwide Hackathon</span>
        </p>
      </div>
      
      <nav class="flex items-center gap-8 text-[12px] font-bold uppercase tracking-widest text-slate-400">
        <NuxtLink to="/legal" class="hover:text-slate-900 transition-colors">Legal Info</NuxtLink>
        <a href="https://github.com/cmoremore/pratrol" target="_blank" class="hover:text-slate-900 transition-colors">GitHub</a>
        <span class="cursor-default">© 2026 STRATORYS SAS</span>
      </nav>
    </div>
  </footer>
</template>

<script setup lang="ts">
// No props needed
</script>
