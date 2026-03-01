<script setup lang="ts">
const { data: posts } = await useAsyncData('blog-posts', () => 
  queryCollection('blog')
    .order('date', 'DESC')
    .order('stem', 'ASC')
    .all()
)

const sortedPosts = computed(() => posts.value || [])

function postHref(path: string) {
  return path.endsWith('/') ? path : `${path}/`
}

function formatDate(dateStr: string) {
  const d = new Date(dateStr)
  return d.toLocaleDateString('en-GB', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).replace(',', '')
}
</script>

<template>
  <div class="min-h-screen page-wash">
    <SiteHeader active="blog" />

    <main class="shell pb-24 pt-14 lg:pt-20">
      <header>
        <span class="kicker">Engineering Journal</span>
        <h1 class="mt-5 text-4xl font-bold tracking-tight text-slate-900 sm:text-5xl lg:text-6xl">
          Notes on prioritized triage.
        </h1>
        <p class="mt-6 text-xl leading-relaxed text-slate-600">
          A collection of rollout playbooks, technical reasoning, and maintainer workflow patterns 
          built on top of Mistral-powered analysis.
        </p>
      </header>

      <section class="mt-20 grid gap-1 border-t border-slate-200 pt-12">
        <NuxtLink 
          v-for="post in sortedPosts" 
          :key="post.path" 
          :to="postHref(post.path)"
          class="group block panel p-8 hover:bg-slate-50/50"
        >
          <div class="flex flex-col md:flex-row md:items-start md:justify-between gap-8">
            <div class="flex-grow">
              <div class="flex items-center gap-4 mb-4">
                <span class="text-[10px] font-bold uppercase tracking-[0.15em] text-slate-900 px-2 py-0.5 border border-slate-200 bg-slate-50">{{ post.category }}</span>
                <span class="text-[11px] font-bold text-slate-400 uppercase tracking-widest">{{ formatDate(post.date) }}</span>
              </div>
              <h2 class="text-2xl font-bold text-slate-900 group-hover:text-black leading-tight">
                {{ post.title }}
              </h2>
              <p class="mt-4 text-slate-600 text-sm leading-relaxed line-clamp-2">
                {{ post.description }}
              </p>
            </div>
            
            <div class="hidden md:flex items-center text-slate-900 font-bold text-xs uppercase tracking-[0.2em] gap-2 opacity-0 group-hover:opacity-100 whitespace-nowrap">
              Read Article
              <span class="text-xl">→</span>
            </div>
          </div>
        </NuxtLink>
      </section>

      <section class="mt-20 grid-wash p-12 border border-slate-200 text-center bg-white/50 backdrop-blur-sm">
        <h3 class="text-lg font-bold text-slate-900">Have questions about rollout?</h3>
        <p class="mt-2 text-sm text-slate-600">Our playbooks are built from real team feedback. Join the discussion on GitHub to share your workflow.</p>
        <div class="mt-8">
          <a href="https://github.com/cmoremore/pratrol" class="btn-secondary px-8">Contribute on GitHub</a>
        </div>
      </section>
    </main>
  </div>
</template>
