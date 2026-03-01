<template>
    <div
        class="border border-[#d0d7de] bg-white shadow-sm font-sans text-[14px] leading-[1.5]"
    >
        <div
            class="flex items-center justify-between bg-[#f6f8fa] px-4 py-2 border-b border-[#d0d7de]"
        >
            <div class="flex items-center gap-2">
                <div
                    class="w-5 h-5 bg-[#0f172a] flex items-center justify-center text-[10px] text-white font-bold rounded-sm"
                >
                    P
                </div>
                <span class="font-semibold text-[#1f2328]">pratrol-bot</span>
                <span class="text-[#636c76]">commented 2m ago</span>
            </div>
            <div
                class="px-1.5 py-0.5 border border-[#d0d7de] bg-white text-[#636c76] text-[12px] font-medium"
            >
                Contributor
            </div>
        </div>

        <div class="p-4 text-[#1f2328]">
            <div class="mb-4">
                <h3
                    class="text-base font-semibold mb-2 flex items-center gap-2"
                >
                    <span class="w-2 h-2 rounded-full bg-amber-500"></span>
                    Triage Brief: Medium Confidence
                </h3>
                <p class="text-[#636c76] mb-3">
                    Pratrol has assessed this change across three primary risk
                    vectors.
                </p>
            </div>

            <div
                class="overflow-hidden border border-[#d0d7de] rounded-md mb-4"
            >
                <table class="w-full text-left border-collapse">
                    <thead class="bg-[#f6f8fa] border-b border-[#d0d7de]">
                        <tr>
                            <th class="px-3 py-2 font-semibold">Risk Vector</th>
                            <th class="px-3 py-2 font-semibold">Score</th>
                            <th class="px-3 py-2 font-semibold text-right">
                                Tier
                            </th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-[#d0d7de]">
                        <tr>
                            <td class="px-3 py-2">Contributor Context</td>
                            <td class="px-3 py-2">72 / 100</td>
                            <td
                                class="px-3 py-2 text-right font-medium text-emerald-600"
                            >
                                High
                            </td>
                        </tr>
                        <tr>
                            <td class="px-3 py-2 text-[#cf222e]">
                                Logic Risk (Mistral)
                            </td>
                            <td class="px-3 py-2 text-[#cf222e]">42 / 100</td>
                            <td
                                class="px-3 py-2 text-right font-medium text-red-600"
                            >
                                Low
                            </td>
                        </tr>
                        <tr class="bg-[#f6f8fa] font-bold">
                            <td class="px-3 py-2">Overall Confidence</td>
                            <td class="px-3 py-2">57 / 100</td>
                            <td class="px-3 py-2 text-right text-amber-600">
                                Medium
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div
                class="space-y-2 text-[13px] border-l-2 border-[#afb8c1] pl-3 py-1 text-[#636c76]"
            >
                <p>
                    <strong class="text-[#1f2328]">Key Signal:</strong> First
                    contribution to the
                    <code class="bg-[#afb8c1]/20 px-1 font-mono"
                        >auth-service</code
                    >
                    module.
                </p>
                <p>
                    <strong class="text-[#1f2328]">Logic Risk:</strong> Mistral
                    flagged a potential race condition in
                    <code class="bg-[#afb8c1]/20 px-1 font-mono"
                        >SessionStore.ts</code
                    >
                    line 142.
                </p>
                <p>
                    <strong class="text-[#1f2328]">Recommendation:</strong>
                    Verify retry logic before merging.
                </p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts"></script>
