<template>
  <div class="min-h-screen page-wash">
    <SiteHeader active="home" />

    <main class="shell pb-24 pt-14 lg:pt-20">
      <!-- Hero Section -->
      <section class="grid gap-12 lg:grid-cols-[1.2fr,1fr] lg:items-center">
        <div>
          <span class="kicker">GitHub Marketplace App</span>
          <h1 class="mt-5 text-4xl font-bold leading-[1.1] tracking-tight text-slate-950 sm:text-5xl lg:text-6xl">
            Triage pull requests by risk, not by timestamp.
          </h1>
          <p class="mt-6 max-w-2xl text-lg leading-relaxed text-slate-700">
            Pratrol analyzes incoming pull requests and posts a concise confidence brief directly in the thread. 
            Spend 30 minutes where it matters; scan the rest in 3.
          </p>
          <div class="mt-8 flex flex-wrap gap-4">
            <a id="install" href="#" class="btn-primary">Install on GitHub</a>
            <NuxtLink to="/scoring" class="btn-secondary">Scoring Methodology</NuxtLink>
          </div>

          <div class="mt-10 border-l-2 border-slate-200 pl-6">
            <p class="text-xs font-semibold uppercase tracking-widest text-slate-400">Powered by</p>
            <div class="mt-3 flex items-center gap-4">
              <MistralLogo classes="h-6 w-auto grayscale opacity-70" />
              <p class="text-sm font-medium text-slate-600">Mistral AI for deep logic assessment</p>
            </div>
          </div>
        </div>

        <div class="relative">
          <div class="absolute -inset-4 grid-wash opacity-40"></div>
          <div class="relative shadow-2xl">
            <PratrolBrief />
          </div>
        </div>
      </section>

      <!-- How it Works / Workflow -->
      <section class="mt-32">
        <div class="max-w-3xl">
          <span class="kicker">Workflow</span>
          <h2 class="section-title mt-4 text-3xl sm:text-4xl">Integrated directly into your review loop.</h2>
          <p class="mt-4 text-lg text-slate-600">
            No new dashboards. No context switching. Pratrol works where your team already lives.
          </p>
        </div>

        <div class="mt-12 grid gap-1 border-t border-slate-200 pt-12 sm:grid-cols-3">
          <div class="panel p-6">
            <div class="text-xs font-bold text-blue-600 uppercase tracking-widest mb-4">01. Listen</div>
            <h3 class="text-lg font-semibold mb-2">Webhook Trigger</h3>
            <p class="text-sm text-slate-600 leading-relaxed">
              Pratrol hooks into <code class="bg-slate-100 px-1 font-mono text-[13px]">pull_request.opened</code> events. Analysis starts instantly.
            </p>
          </div>
          <div class="panel p-6">
            <div class="text-xs font-bold text-blue-600 uppercase tracking-widest mb-4">02. Analyze</div>
            <h3 class="text-lg font-semibold mb-2">Multi-Vector Check</h3>
            <p class="text-sm text-slate-600 leading-relaxed">
              We weigh contributor history against file sensitivity and Mistral-powered logic risk analysis.
            </p>
          </div>
          <div class="panel p-6">
            <div class="text-xs font-bold text-blue-600 uppercase tracking-widest mb-4">03. Signal</div>
            <h3 class="text-lg font-semibold mb-2">Actionable Brief</h3>
            <p class="text-sm text-slate-600 leading-relaxed">
              A structured triage brief is posted as a PR comment, highlighting specific areas for concern.
            </p>
          </div>
        </div>
      </section>

      <!-- The Contrast / Value Prop -->
      <section class="mt-32">
        <div class="grid gap-12 lg:grid-cols-2">
          <div>
            <span class="kicker">The Signal</span>
            <h2 class="section-title mt-4 text-3xl">Review depth that aligns with risk.</h2>
            <p class="mt-4 text-slate-600 leading-relaxed">
              Most pull requests are safe, routine updates. Others touch sensitive infrastructure or include complex logic traps. 
              Pratrol helps your senior engineers focus their attention where it's actually needed.
            </p>
            <ul class="mt-8 space-y-4">
              <li class="flex items-start gap-3">
                <span class="mt-1 flex h-5 w-5 shrink-0 items-center justify-center bg-blue-100 text-blue-700 font-bold text-[10px]">✓</span>
                <span class="text-sm text-slate-700 font-medium">Detect logic risks that static analysis tools miss.</span>
              </li>
              <li class="flex items-start gap-3">
                <span class="mt-1 flex h-5 w-5 shrink-0 items-center justify-center bg-blue-100 text-blue-700 font-bold text-[10px]">✓</span>
                <span class="text-sm text-slate-700 font-medium">Identify first-time contributors in sensitive modules.</span>
              </li>
              <li class="flex items-start gap-3">
                <span class="mt-1 flex h-5 w-5 shrink-0 items-center justify-center bg-blue-100 text-blue-700 font-bold text-[10px]">✓</span>
                <span class="text-sm text-slate-700 font-medium">Standardize triage across multiple repositories.</span>
              </li>
            </ul>
          </div>

          <div class="grid gap-4">
            <article class="panel p-8">
              <h3 class="text-lg font-semibold mb-4 text-slate-400">Without Pratrol</h3>
              <p class="text-sm text-slate-500 line-through decoration-slate-300">"Every PR gets the same surface-level scan because we're underwater."</p>
            </article>
            <article class="panel border-blue-200 bg-blue-50/30 p-8">
              <h3 class="text-lg font-semibold mb-4 text-blue-900">With Pratrol</h3>
              <p class="text-sm text-slate-700 italic">"I see a Medium Confidence brief on an Auth file and immediately know I need to spend 20 minutes on it."</p>
            </article>
          </div>
        </div>
      </section>

      <!-- Footer CTA -->
      <section class="mt-32 border-t border-slate-200 pt-20 text-center">
        <h2 class="section-title text-3xl sm:text-4xl">Ready to automate your triage?</h2>
        <p class="mt-6 text-lg text-slate-600 max-w-2xl mx-auto">
          Install the GitHub app and get your first confidence brief in minutes. 
          Simple setup, no configuration required.
        </p>
        <div class="mt-10 flex flex-wrap justify-center gap-4">
          <a href="#" class="btn-primary px-12 py-4">Install Pratrol</a>
          <NuxtLink to="/scoring" class="btn-secondary px-12 py-4">Read Technical Docs</NuxtLink>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import PratrolBrief from '~/components/PratrolBrief.vue'
</script>
