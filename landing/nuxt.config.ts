export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  app: {
    head: {
      title: 'Pratrol - Prioritized Pull Request Triage',
      meta: [
        { name: 'description', content: 'Triage pull requests by risk, not by timestamp. AI-assisted triage for GitHub teams.' },
        { name: 'theme-color', content: '#f8fafc' }
      ],
      link: [
        { rel: 'icon', type: 'image/png', href: '/logo.png' }
      ]
    }
  },
  modules: ['@nuxtjs/tailwindcss', '@nuxt/content'],
  css: ['~/assets/css/main.css'],
  nitro: {
    preset: 'static',
  },
  routeRules: {
    '/**': { prerender: true },
  },
})
