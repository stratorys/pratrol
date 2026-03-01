<template>
    <div class="min-h-screen bg-gradient-to-b from-slate-50 via-white to-white">
        <SiteHeader active="home" />

        <main class="max-w-7xl mx-auto px-8 pb-24 pt-14 lg:pt-20">
            <section
                class="grid gap-12 lg:grid-cols-[1.2fr,1fr] lg:items-center"
            >
                <div>
                    <span
                        class="inline-block border border-slate-200 bg-slate-50 text-slate-500 text-[0.7rem] font-bold tracking-widest uppercase px-2.5 py-1"
                        >GitHub Marketplace App</span
                    >
                    <h1
                        class="mt-5 text-4xl font-bold leading-[1.1] tracking-tight text-slate-950 sm:text-5xl lg:text-6xl"
                    >
                        Triage pull requests by risk, not by timestamp.
                    </h1>
                    <p class="mt-6 text-lg leading-relaxed text-slate-700">
                        Pratrol analyzes incoming pull requests and posts a
                        concise confidence brief directly in the thread. Spend
                        30 minutes where it matters; scan the rest in 3.
                    </p>
                    <div class="mt-8 flex flex-wrap gap-4">
                        <a
                            id="install"
                            href="https://github.com/apps/pratrol"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="border border-slate-900 bg-slate-900 text-white font-semibold text-sm px-5 py-2.5 hover:bg-black hover:border-black"
                            >Install on GitHub</a
                        >
                        <NuxtLink
                            to="/scoring"
                            class="border border-slate-200 text-slate-700 font-semibold text-sm px-5 py-2.5 bg-white hover:bg-slate-50 hover:border-slate-300"
                            >Scoring Methodology</NuxtLink
                        >
                    </div>

                    <div class="mt-10 border-l-2 border-slate-200 pl-6">
                        <p
                            class="text-xs font-semibold uppercase tracking-widest text-slate-400"
                        >
                            Powered by
                        </p>
                        <div class="mt-3 flex items-center gap-4">
                            <MistralLogo
                                classes="h-6 w-auto grayscale opacity-70"
                            />
                            <p class="text-sm font-medium text-slate-600">
                                Mistral AI for deep logic assessment
                            </p>
                        </div>
                    </div>
                </div>

                <div class="relative">
                    <div
                        class="absolute -inset-4 opacity-40"
                        style="
                            background-image:
                                linear-gradient(
                                    to right,
                                    rgba(148, 163, 184, 0.1) 1px,
                                    transparent 1px
                                ),
                                linear-gradient(
                                    to bottom,
                                    rgba(148, 163, 184, 0.1) 1px,
                                    transparent 1px
                                );
                            background-size: 32px 32px;
                        "
                    ></div>
                    <div class="relative shadow-2xl">
                        <PratrolBrief />
                    </div>
                </div>
            </section>

            <section class="mt-32">
                <div>
                    <span
                        class="inline-block border border-slate-200 bg-slate-50 text-slate-500 text-[0.7rem] font-bold tracking-widest uppercase px-2.5 py-1"
                        >Workflow</span
                    >
                    <h2
                        class="mt-4 text-3xl sm:text-4xl text-slate-900 font-bold tracking-tight"
                    >
                        Integrated directly into your review loop.
                    </h2>
                    <p class="mt-4 text-lg text-slate-600">
                        No new dashboards. No context switching. Pratrol works
                        where your team already lives.
                    </p>
                </div>

                <div
                    class="mt-12 grid gap-1 border-t border-slate-200 pt-12 sm:grid-cols-3"
                >
                    <div class="bg-white border border-slate-200 shadow-sm p-6">
                        <div
                            class="text-xs font-bold text-slate-900 uppercase tracking-widest mb-4"
                        >
                            01. Listen
                        </div>
                        <h3 class="text-lg font-semibold mb-2">
                            Webhook Trigger
                        </h3>
                        <p class="text-sm text-slate-600 leading-relaxed">
                            Pratrol hooks into
                            <code
                                class="bg-slate-100 px-1 font-mono text-[13px]"
                                >pull_request.opened</code
                            >
                            events. Analysis starts instantly.
                        </p>
                    </div>
                    <div class="bg-white border border-slate-200 shadow-sm p-6">
                        <div
                            class="text-xs font-bold text-slate-900 uppercase tracking-widest mb-4"
                        >
                            02. Analyze
                        </div>
                        <h3 class="text-lg font-semibold mb-2">
                            Multi-Vector Check
                        </h3>
                        <p class="text-sm text-slate-600 leading-relaxed">
                            We weigh contributor history against file
                            sensitivity and Mistral-powered logic risk analysis.
                        </p>
                    </div>
                    <div class="bg-white border border-slate-200 shadow-sm p-6">
                        <div
                            class="text-xs font-bold text-slate-900 uppercase tracking-widest mb-4"
                        >
                            03. Signal
                        </div>
                        <h3 class="text-lg font-semibold mb-2">
                            Actionable Brief
                        </h3>
                        <p class="text-sm text-slate-600 leading-relaxed">
                            A structured triage brief is posted as a PR comment,
                            highlighting specific areas for concern.
                        </p>
                    </div>
                </div>
            </section>

            <section class="mt-32">
                <div class="grid gap-12 lg:grid-cols-2">
                    <div>
                        <span
                            class="inline-block border border-slate-200 bg-slate-50 text-slate-500 text-[0.7rem] font-bold tracking-widest uppercase px-2.5 py-1"
                            >The Signal</span
                        >
                        <h2
                            class="mt-4 text-3xl text-slate-900 font-bold tracking-tight"
                        >
                            Review depth that aligns with risk.
                        </h2>
                        <p class="mt-4 text-slate-600 leading-relaxed">
                            Most pull requests are safe, routine updates. Others
                            touch sensitive infrastructure or include complex
                            logic traps. Pratrol helps your senior engineers
                            focus their attention where it's actually needed.
                        </p>
                        <ul class="mt-8 space-y-4">
                            <li class="flex items-start gap-3">
                                <span
                                    class="mt-1 flex h-5 w-5 shrink-0 items-center justify-center bg-slate-100 text-slate-900 font-bold text-[10px]"
                                    >✓</span
                                >
                                <span class="text-sm text-slate-700 font-medium"
                                    >Detect logic risks that static analysis
                                    tools miss.</span
                                >
                            </li>
                            <li class="flex items-start gap-3">
                                <span
                                    class="mt-1 flex h-5 w-5 shrink-0 items-center justify-center bg-slate-100 text-slate-900 font-bold text-[10px]"
                                    >✓</span
                                >
                                <span class="text-sm text-slate-700 font-medium"
                                    >Identify first-time contributors in
                                    sensitive modules.</span
                                >
                            </li>
                            <li class="flex items-start gap-3">
                                <span
                                    class="mt-1 flex h-5 w-5 shrink-0 items-center justify-center bg-slate-100 text-slate-900 font-bold text-[10px]"
                                    >✓</span
                                >
                                <span class="text-sm text-slate-700 font-medium"
                                    >Standardize triage across multiple
                                    repositories.</span
                                >
                            </li>
                        </ul>
                    </div>

                    <div class="grid gap-4">
                        <article
                            class="bg-white border border-slate-200 shadow-sm p-8"
                        >
                            <h3
                                class="text-lg font-semibold mb-4 text-slate-400"
                            >
                                Without Pratrol
                            </h3>
                            <p
                                class="text-sm text-slate-500 line-through decoration-slate-300"
                            >
                                "Every PR gets the same surface-level scan
                                because we're underwater."
                            </p>
                        </article>
                        <article
                            class="bg-white shadow-sm border border-slate-900 bg-slate-50/30 p-8"
                        >
                            <h3
                                class="text-lg font-semibold mb-4 text-slate-900"
                            >
                                With Pratrol
                            </h3>
                            <p class="text-sm text-slate-700 italic">
                                "I see a Medium Confidence brief on an Auth file
                                and immediately know I need to spend 20 minutes
                                on it."
                            </p>
                        </article>
                    </div>
                </div>
            </section>

            <section class="mt-32 border-t border-slate-200 pt-20 text-center">
                <h2
                    class="text-3xl sm:text-4xl text-slate-900 font-bold tracking-tight"
                >
                    Ready to automate your triage?
                </h2>
                <p class="mt-6 text-lg text-slate-600 mx-auto">
                    Install the GitHub app and get your first confidence brief
                    in minutes. Simple setup, no configuration required.
                </p>
                <div class="mt-10 flex flex-wrap justify-center gap-4">
                    <a
                        href="https://github.com/apps/pratrol"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="border border-slate-900 bg-slate-900 text-white font-semibold text-sm px-12 py-4 hover:bg-black hover:border-black"
                        >Install Pratrol</a
                    >
                    <NuxtLink
                        to="/scoring"
                        class="border border-slate-200 text-slate-700 font-semibold text-sm px-12 py-4 bg-white hover:bg-slate-50 hover:border-slate-300"
                        >Read Technical Docs</NuxtLink
                    >
                </div>
            </section>
        </main>
    </div>
</template>

<script setup lang="ts">
import PratrolBrief from "~/components/PratrolBrief.vue";
</script>
