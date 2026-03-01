<template>
  <div class="flex flex-col min-h-screen">
    <NuxtRouteAnnouncer />
    <div class="flex-grow">
      <NuxtPage />
    </div>
    <SiteFooter />
  </div>
</template>

<script setup lang="ts">
import SiteFooter from '~/components/SiteFooter.vue'
</script>
