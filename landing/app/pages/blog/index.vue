<script setup lang="ts">
import SiteHeader from '~/components/SiteHeader.vue'

const { data: posts } = await useAsyncData('blog-posts', () => queryCollection('blog').all())

const sortedPosts = computed(() => {
  return [...(posts.value || [])].sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
})
</script>

<template>
  <div class="min-h-screen page-wash">
    <SiteHeader active="blog" />

    <main class="shell pb-24 pt-14 lg:pt-20">
      <header class="max-w-3xl">
        <span class="kicker">Engineering Journal</span>
        <h1 class="mt-5 text-4xl font-bold tracking-tight text-slate-950 sm:text-5xl lg:text-6xl">
          Notes on prioritized triage.
        </h1>
        <p class="mt-6 text-xl leading-relaxed text-slate-700">
          A collection of rollout playbooks, technical reasoning, and maintainer workflow patterns 
          built on top of Mistral-powered analysis.
        </p>
      </header>

      <section class="mt-20 grid gap-1 border-t border-slate-200 pt-12">
        <NuxtLink 
          v-for="post in sortedPosts" 
          :key="post.path" 
          :to="post.path"
          class="group block panel p-8 hover:bg-slate-50/50 transition-colors"
        >
          <div class="flex flex-col md:flex-row md:items-start md:justify-between gap-6">
            <div class="max-w-2xl">
              <div class="flex items-center gap-3 mb-4">
                <span class="text-xs font-bold uppercase tracking-widest text-blue-600 px-2 py-0.5 border border-blue-100 bg-blue-50/50">{{ post.category }}</span>
                <span class="text-xs font-bold text-slate-400 uppercase tracking-widest">{{ post.date }}</span>
              </div>
              <h2 class="text-2xl font-bold text-slate-950 group-hover:text-blue-700 transition-colors leading-tight">
                {{ post.title }}
              </h2>
              <p class="mt-4 text-slate-600 text-sm leading-relaxed line-clamp-2">
                {{ post.description }}
              </p>
            </div>
            
            <div class="hidden md:flex items-center text-blue-600 font-bold text-xs uppercase tracking-widest gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
              Read Article
              <span class="text-lg">→</span>
            </div>
          </div>
        </NuxtLink>
      </section>

      <!-- Sidebar-like CTA for the blog index bottom -->
      <section class="mt-20 grid-wash p-12 border border-slate-100 text-center">
        <h3 class="text-lg font-bold text-slate-950">Have questions about rollout?</h3>
        <p class="mt-2 text-sm text-slate-600">Our playbooks are built from real team feedback. Join the discussion on GitHub.</p>
        <div class="mt-8">
          <a href="https://github.com/cmoremore/pratrol" class="btn-secondary px-8">Contribute on GitHub</a>
        </div>
      </section>
    </main>
  </div>
</template>
