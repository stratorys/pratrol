export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  modules: ['@nuxtjs/tailwindcss', '@nuxt/content'],
  css: ['~/assets/css/main.css'],
  nitro: {
    preset: 'static',
  },
  routeRules: {
    '/**': { prerender: true },
  },
})
