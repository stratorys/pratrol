<script setup lang="ts">
const route = useRoute()
const slug = String(route.params.slug)

const { data: post } = await useAsyncData(`blog-${slug}`, async () => {
  const byStem = await queryCollection('blog').where('stem', '=', `blog/${slug}`).first()
  if (byStem) return byStem

  const byPath = await queryCollection('blog').path(`/blog/${slug}`).first()
  if (byPath) return byPath

  return await queryCollection('blog').path(`/blog/${slug}/`).first()
})

if (!post.value && process.server) {
  throw createError({ statusCode: 404, statusMessage: 'Post not found' })
}

function formatDate(dateStr: string) {
  if (!dateStr) return ''
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
      <div v-if="post">
        <article class="w-full">
          <header class="mb-16">
            <div class="flex items-center gap-4 mb-6">
              <span class="text-[10px] font-bold uppercase tracking-[0.15em] text-slate-900 px-2 py-0.5 border border-slate-200 bg-slate-50">{{ post.category }}</span>
              <span class="text-[11px] font-bold text-slate-400 uppercase tracking-widest">{{ formatDate(post.date) }}</span>
            </div>
            <h1 class="text-4xl md:text-5xl font-bold tracking-tight text-slate-900 leading-tight">
              {{ post.title }}
            </h1>
            <p class="mt-8 text-xl text-slate-600 leading-relaxed font-medium border-l-4 border-slate-900 pl-6">
              {{ post.description }}
            </p>
            
            <div class="mt-12 flex items-center gap-4 border-t border-slate-100 pt-8">
              <div class="w-10 h-10 bg-slate-900 flex items-center justify-center text-white font-bold text-xs">P</div>
              <div class="text-[13px]">
                <p class="font-bold text-slate-900">{{ post.author }}</p>
                <p class="text-slate-500 uppercase tracking-widest text-[10px] font-bold mt-0.5">Pratrol Engineering Team</p>
              </div>
            </div>
          </header>

          <div class="panel p-8 md:p-12 shadow-sm bg-white border border-slate-100 prose prose-slate max-w-none">
            <ContentRenderer :value="post" />
          </div>

          <div class="mt-16 flex items-center justify-between border-t border-slate-100 pt-8">
            <NuxtLink to="/blog" class="text-xs font-bold uppercase tracking-[0.2em] text-slate-400 hover:text-slate-900 flex items-center gap-2">
              <span class="text-xl">←</span>
              Back to Journal
            </NuxtLink>
            
            <div class="flex items-center gap-4">
              <span class="text-[10px] font-bold uppercase tracking-widest text-slate-400">Share article</span>
              <div class="flex gap-2">
                <a href="#" class="w-8 h-8 flex items-center justify-center border border-slate-200 text-slate-400 hover:text-slate-900">𝕏</a>
                <a href="#" class="w-8 h-8 flex items-center justify-center border border-slate-200 text-slate-400 hover:text-slate-900">in</a>
              </div>
            </div>
          </div>
        </article>
      </div>
      <div v-else class="text-center py-20">
        <p class="text-slate-500 text-sm font-bold uppercase tracking-widest">Post not found</p>
        <NuxtLink to="/blog" class="btn-secondary mt-8 inline-block">Back to Journal</NuxtLink>
      </div>
    </main>
  </div>
</template>
