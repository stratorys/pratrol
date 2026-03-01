<script setup lang="ts">
const { data: posts } = await useAsyncData('blog-posts', () => queryCollection('blog').all())

const sortedPosts = computed(() => {
  return [...(posts.value || [])].sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
})
</script>

<template>
  <div class="min-h-screen page-wash">
    <SiteHeader active="blog" />

    <main class="shell pb-24 pt-14 lg:pt-20">
      <section class="grid gap-10 lg:grid-cols-[1.6fr,1fr] lg:items-end">
        <div>
          <span class="kicker">Maintainer Journal</span>
          <h1 class="mt-5 text-4xl font-semibold tracking-tight text-slate-950 sm:text-5xl">Blog</h1>
          <p class="mt-4 max-w-3xl text-lg text-slate-700">
            Practical notes for teams improving pull request review flow with Mistral AI-assisted triage.
          </p>
        </div>
        <aside class="panel p-5">
          <p class="text-xs font-semibold uppercase tracking-wide text-slate-500">About this blog</p>
          <p class="mt-3 text-sm text-slate-700">
            Rollout playbooks, scoring-tuning notes, and reviewer workflow patterns from real engineering teams.
          </p>
        </aside>
      </section>

      <section class="mt-12 grid gap-4">
        <article v-for="post in sortedPosts" :key="post.path" class="panel p-6">
          <p class="text-xs font-semibold uppercase tracking-[0.16em] text-blue-700">{{ post.category }}</p>
          <h2 class="mt-2 text-2xl font-semibold text-slate-950">{{ post.title }}</h2>
          <p class="mt-3 text-slate-700">{{ post.description }}</p>
          <div class="mt-4 flex items-center justify-between text-sm text-slate-500">
            <span>{{ post.date }}</span>
            <NuxtLink :to="post.path" class="font-semibold text-blue-700 hover:text-blue-800">Read article</NuxtLink>
          </div>
        </article>
      </section>
    </main>
  </div>
</template>
