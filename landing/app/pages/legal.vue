<template>
  <div class="min-h-screen page-wash">
    <SiteHeader active="home" />

    <main class="shell pb-24 pt-14 lg:pt-20">
      <section class="max-w-3xl">
        <span class="kicker">Legal & Transparency</span>
        <h1 class="mt-5 text-4xl font-bold tracking-tight text-slate-950">Legal Information</h1>
        <p class="mt-4 text-lg text-slate-600">
          This project is developed by <span class="font-semibold text-slate-900">STRATORYS SAS</span>.
        </p>
      </section>

      <div class="mt-16 grid gap-12 sm:grid-cols-2">
        <div class="panel p-8">
          <h2 class="text-xs font-bold uppercase tracking-widest text-blue-600 mb-6 underline underline-offset-8 decoration-2 decoration-blue-100">The Entity</h2>
          <div class="space-y-4 text-sm text-slate-700 leading-relaxed">
            <p><strong class="text-slate-900 block font-semibold mb-1">Company:</strong> STRATORYS SAS</p>
            <p><strong class="text-slate-900 block font-semibold mb-1">Share Capital:</strong> 100 EUR</p>
            <p><strong class="text-slate-900 block font-semibold mb-1">Registration:</strong> RCS Paris 989 015 375</p>
            <p><strong class="text-slate-900 block font-semibold mb-1">Address:</strong> 40 Rue Alexandre Dumas, 75011 Paris, France</p>
            <p><strong class="text-slate-900 block font-semibold mb-1">Contact:</strong> contact@stratorys.com</p>
          </div>
        </div>

        <div class="panel p-8">
          <h2 class="text-xs font-bold uppercase tracking-widest text-blue-600 mb-6 underline underline-offset-8 decoration-2 decoration-blue-100">Hosting & Privacy</h2>
          <div class="space-y-4 text-sm text-slate-700 leading-relaxed">
            <p><strong class="text-slate-900 block font-semibold mb-1">Provider:</strong> Cloudflare, Inc.</p>
            <p><strong class="text-slate-900 block font-semibold mb-1">Address:</strong> 101 Townsend St, San Francisco, CA 94107, USA</p>
            <p><strong class="text-slate-900 block font-semibold mb-1">Cookies:</strong> None used.</p>
            <p><strong class="text-slate-900 block font-semibold mb-1">Privacy:</strong> Email collected via contact form is only used to respond. No data is shared with third parties.</p>
          </div>
        </div>
      </div>

      <section class="mt-16 panel p-8">
        <h2 class="text-xs font-bold uppercase tracking-widest text-blue-600 mb-6 underline underline-offset-8 decoration-2 decoration-blue-100">Intellectual Property</h2>
        <div class="prose max-w-none text-sm leading-relaxed text-slate-700">
          <p>
            All content (text, logos, code) is the exclusive property of STRATORYS SAS. "STRATORYS" is a registered trademark.
          </p>
          <p class="mt-4">
            Research and projects are distributed under specific licenses (MIT or MPL-2.0) as noted in their respective repositories.
          </p>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import SiteHeader from '~/components/SiteHeader.vue'
</script>
