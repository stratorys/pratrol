<template>
    <div class="min-h-screen bg-gradient-to-b from-slate-50 via-white to-white">
        <SiteHeader active="home" />

        <main class="max-w-7xl mx-auto px-8 pb-24 pt-14 lg:pt-20">
            <section>
                <span
                    class="inline-block border border-slate-200 bg-slate-50 text-slate-500 text-[0.7rem] font-bold tracking-widest uppercase px-2.5 py-1"
                    >Legal & Transparency</span
                >
                <h1
                    class="mt-5 text-4xl font-bold tracking-tight text-slate-950"
                >
                    Legal Information
                </h1>
                <p class="mt-4 text-lg text-slate-600">
                    This project is developed by
                    <span class="font-semibold text-slate-900"
                        >STRATORYS SAS</span
                    >.
                </p>
            </section>

            <div class="mt-16 grid gap-12 sm:grid-cols-2">
                <div class="bg-white border border-slate-200 shadow-sm p-8">
                    <h2
                        class="text-xs font-bold uppercase tracking-widest text-slate-900 mb-6 underline underline-offset-8 decoration-2 decoration-slate-100"
                    >
                        The Entity
                    </h2>
                    <div
                        class="space-y-4 text-sm text-slate-700 leading-relaxed"
                    >
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Company:</strong
                            >
                            STRATORYS SAS
                        </p>
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Share Capital:</strong
                            >
                            100 EUR
                        </p>
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Registration:</strong
                            >
                            RCS Paris 989 015 375
                        </p>
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Address:</strong
                            >
                            40 Rue Alexandre Dumas, 75011 Paris, France
                        </p>
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Contact:</strong
                            >
                            contact@stratorys.com
                        </p>
                    </div>
                </div>

                <div class="bg-white border border-slate-200 shadow-sm p-8">
                    <h2
                        class="text-xs font-bold uppercase tracking-widest text-slate-900 mb-6 underline underline-offset-8 decoration-2 decoration-slate-100"
                    >
                        Hosting & Privacy
                    </h2>
                    <div
                        class="space-y-4 text-sm text-slate-700 leading-relaxed"
                    >
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Provider:</strong
                            >
                            Cloudflare, Inc.
                        </p>
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Address:</strong
                            >
                            101 Townsend St, San Francisco, CA 94107, USA
                        </p>
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Cookies:</strong
                            >
                            None used.
                        </p>
                        <p>
                            <strong
                                class="text-slate-900 block font-semibold mb-1"
                                >Privacy:</strong
                            >
                            Email collected via contact form is only used to
                            respond. No data is shared with third parties.
                        </p>
                    </div>
                </div>
            </div>

            <section
                class="mt-16 bg-white border border-slate-200 shadow-sm p-8"
            >
                <h2
                    class="text-xs font-bold uppercase tracking-widest text-slate-900 mb-6 underline underline-offset-8 decoration-2 decoration-slate-100"
                >
                    Intellectual Property
                </h2>
                <div
                    class="prose prose-slate max-w-none text-sm leading-relaxed text-slate-700"
                >
                    <p>
                        All content (text, logos, code) is the exclusive
                        property of STRATORYS SAS. "STRATORYS" is a registered
                        trademark.
                    </p>
                    <p class="mt-4">
                        Research and projects are distributed under the MPL-2.0
                        license as noted in their respective repositories.
                    </p>
                </div>
            </section>
        </main>
    </div>
</template>

<script setup lang="ts">
import SiteHeader from "~/components/SiteHeader.vue";
</script>
