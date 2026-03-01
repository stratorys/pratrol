<template>
  <div class="min-h-screen page-wash">
    <SiteHeader active="scoring" />

    <main class="shell pb-24 pt-14 lg:pt-20">
      <section>
        <span class="kicker">Technical Reference</span>
        <h1 class="mt-5 text-4xl font-bold leading-tight tracking-tight text-slate-950 sm:text-5xl lg:text-6xl">
          The Confidence Model: How we weigh PR risk.
        </h1>
        <p class="mt-6 text-xl leading-relaxed text-slate-700">
          Pratrol transforms raw pull request data into actionable triage signals. 
          By weighing contributor history against change complexity, we provide a structured first-pass assessment for every PR.
        </p>
      </section>

      <section class="mt-24 grid gap-12 lg:grid-cols-2">
        <div>
          <h2 class="section-title text-2xl mb-8">Primary Risk Vectors</h2>
          <div class="space-y-6">
            <div class="panel p-6">
              <h3 class="font-bold text-slate-900 mb-2">01. Contributor Context</h3>
              <p class="text-sm text-slate-600 leading-relaxed">
                We analyze the author's history within the repository. Have they touched these specific files before? 
                Are they a frequent contributor or a first-timer in a sensitive module?
              </p>
            </div>
            <div class="panel p-6 border-l-4 border-l-slate-900">
              <h3 class="font-bold text-slate-900 mb-2">02. Logic Risk (Mistral AI)</h3>
              <p class="text-sm text-slate-600 leading-relaxed">
                Mistral AI performs a deep semantic analysis of the diff. Unlike static analysis, it looks for logic traps, 
                race conditions, and architectural misalignments that simple linters miss.
              </p>
            </div>
            <div class="panel p-6">
              <h3 class="font-bold text-slate-900 mb-2">03. File Sensitivity</h3>
              <p class="text-sm text-slate-600 leading-relaxed">
                Changes to <code class="bg-slate-100 px-1 font-mono">auth/</code>, <code class="bg-slate-100 px-1 font-mono">database/</code>, or <code class="bg-slate-100 px-1 font-mono">ci/</code> 
                automatically trigger higher scrutiny weights in the final confidence score.
              </p>
            </div>
          </div>
        </div>

        <div class="panel grid-wash p-8 flex flex-col justify-center">
          <div class="bg-white border border-slate-200 p-6 shadow-sm">
            <p class="text-xs font-bold text-slate-400 uppercase tracking-widest mb-6">Confidence Calculation</p>
            <div class="space-y-4 font-mono text-sm text-slate-700">
              <div class="flex justify-between items-center border-b border-slate-100 pb-2">
                <span>Context Score</span>
                <span class="font-bold">× 0.35</span>
              </div>
              <div class="flex justify-between items-center border-b border-slate-100 pb-2">
                <span>Logic Risk (Mistral)</span>
                <span class="font-bold">× 0.45</span>
              </div>
              <div class="flex justify-between items-center border-b border-slate-100 pb-2">
                <span>File Sensitivity</span>
                <span class="font-bold">× 0.20</span>
              </div>
              <div class="pt-4 flex justify-between items-center text-slate-900 font-bold text-lg">
                <span>Final Confidence</span>
                <span>= Result</span>
              </div>
            </div>
            <p class="mt-8 text-xs text-slate-500 italic">
              * Weights are automatically adjusted based on repository-specific activity patterns.
            </p>
          </div>
        </div>
      </section>

      <section id="tier-actions" class="mt-32">
        <h2 class="section-title text-2xl mb-8">Reviewer Playbook: From Tiers to Action</h2>
        <div class="overflow-hidden border border-slate-200">
          <table class="w-full text-left text-sm">
            <thead class="bg-slate-50 border-b border-slate-200">
              <tr>
                <th class="px-6 py-4 font-bold text-slate-900 uppercase tracking-wider text-[11px]">Tier</th>
                <th class="px-6 py-4 font-bold text-slate-900 uppercase tracking-wider text-[11px]">Signal Indicator</th>
                <th class="px-6 py-4 font-bold text-slate-900 uppercase tracking-wider text-[11px]">Operational Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-200">
              <tr>
                <td class="px-6 py-6 align-top">
                  <span class="inline-flex items-center px-2.5 py-0.5 font-bold text-emerald-700 bg-emerald-50 border border-emerald-100 text-[12px]">HIGH</span>
                </td>
                <td class="px-6 py-6 text-slate-700 leading-relaxed">
                  Trusted contributor profile. Low semantic risk detected in the diff. Standard change patterns.
                </td>
                <td class="px-6 py-6 text-slate-600 font-medium">
                  Proceed with standard peer review. Optimized for speed.
                </td>
              </tr>
              <tr>
                <td class="px-6 py-6 align-top">
                  <span class="inline-flex items-center px-2.5 py-0.5 font-bold text-amber-700 bg-amber-50 border border-amber-100 text-[12px]">MEDIUM</span>
                </td>
                <td class="px-6 py-6 text-slate-700 leading-relaxed">
                  Mixed signals. Potential logic edge cases flagged by Mistral or sensitive file modification.
                </td>
                <td class="px-6 py-6 text-slate-600 font-medium text-slate-900 underline decoration-slate-200 decoration-2 underline-offset-4">
                  Assign a senior reviewer. Validate edge cases before merge.
                </td>
              </tr>
              <tr>
                <td class="px-6 py-6 align-top">
                  <span class="inline-flex items-center px-2.5 py-0.5 font-bold text-rose-700 bg-rose-50 border border-rose-100 text-[12px]">LOW</span>
                </td>
                <td class="px-6 py-6 text-slate-700 leading-relaxed">
                  Critical risk indicators. First-time contribution to sensitive core or logical inconsistencies detected.
                </td>
                <td class="px-6 py-6 text-slate-600 font-medium text-slate-900 underline decoration-slate-200 decoration-2 underline-offset-4">
                  Full architectural review required. Mandatory secondary sign-off.
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section class="mt-32 border-t border-slate-200 pt-20 text-center">
        <h2 class="section-title text-3xl">Apply this model to your workflow.</h2>
        <div class="mt-10 flex flex-wrap justify-center gap-4">
          <a href="#" class="btn-primary px-12 py-4">Install Pratrol on GitHub</a>
          <NuxtLink to="/" class="btn-secondary px-12 py-4">Back to home</NuxtLink>
        </div>
      </section>
    </main>
  </div>
</template>
