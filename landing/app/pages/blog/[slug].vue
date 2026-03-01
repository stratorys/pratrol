<script setup lang="ts">
import SiteHeader from '~/components/SiteHeader.vue'

const route = useRoute()
const path = `/blog/${route.params.slug}`

const { data: post } = await useAsyncData(`blog-${path}`, () => queryCollection('blog').path(path).first())

if (!post.value) {
  throw createError({ statusCode: 404, statusMessage: 'Post not found' })
}
</script>

<template>
  <div class="min-h-screen page-wash">
    <SiteHeader active="blog" />

    <main class="shell pb-24 pt-14 lg:pt-20 flex flex-col items-center">
      <article class="max-w-3xl w-full">
        <header class="mb-16">
          <div class="flex items-center gap-3 mb-6">
            <span class="text-xs font-bold uppercase tracking-widest text-blue-600 px-2 py-0.5 border border-blue-100 bg-blue-50/50">{{ post?.category }}</span>
            <span class="text-xs font-bold text-slate-400 uppercase tracking-widest">{{ post?.date }}</span>
          </div>
          <h1 class="text-4xl md:text-5xl font-bold tracking-tight text-slate-950 leading-tight">
            {{ post?.title }}
          </h1>
          <p class="mt-8 text-xl text-slate-600 leading-relaxed font-medium">
            {{ post?.description }}
          </p>
          
          <div class="mt-8 flex items-center gap-4 border-t border-slate-100 pt-8">
            <div class="w-10 h-10 bg-blue-600 flex items-center justify-center text-white font-bold text-xs">P</div>
            <div class="text-sm">
              <p class="font-bold text-slate-900">{{ post?.author }}</p>
              <p class="text-slate-500">Pratrol Engineering Team</p>
            </div>
          </div>
        </header>

        <div class="panel p-8 md:p-12 shadow-sm bg-white border border-slate-100 prose prose-slate max-w-none">
          <ContentRenderer :value="post" />
        </div>

        <div class="mt-16 flex items-center justify-between border-t border-slate-100 pt-8">
          <NuxtLink to="/blog" class="text-sm font-bold uppercase tracking-widest text-slate-400 hover:text-blue-600 transition-colors flex items-center gap-2">
            <span class="text-lg leading-none">←</span>
            Back to Journal
          </NuxtLink>
          
          <div class="flex items-center gap-4">
            <span class="text-xs font-bold uppercase tracking-widest text-slate-400">Share article</span>
            <div class="flex gap-2">
              <a href="#" class="w-8 h-8 flex items-center justify-center border border-slate-200 text-slate-400 hover:text-blue-600 transition-colors">𝕏</a>
              <a href="#" class="w-8 h-8 flex items-center justify-center border border-slate-200 text-slate-400 hover:text-blue-600 transition-colors">in</a>
            </div>
          </div>
        </div>
      </article>
    </main>
  </div>
</template>
