<script setup lang="ts">
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

    <main class="shell pb-24 pt-14 lg:pt-20">
      <article class="panel p-8">
        <p class="text-xs font-semibold uppercase tracking-[0.16em] text-blue-700">{{ post?.category }}</p>
        <h1 class="mt-3 text-4xl font-semibold tracking-tight text-slate-950">{{ post?.title }}</h1>
        <p class="mt-2 text-sm text-slate-500">{{ post?.date }} · {{ post?.author }}</p>
        <p class="mt-5 text-lg text-slate-700">{{ post?.description }}</p>

        <div class="mt-8 border-t border-slate-200 pt-6 prose prose-slate max-w-none">
          <ContentRenderer :value="post" />
        </div>

        <NuxtLink to="/blog" class="btn-secondary mt-10 inline-block">Back to blog</NuxtLink>
      </article>
    </main>
  </div>
</template>
