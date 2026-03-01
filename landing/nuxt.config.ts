export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  site: {
    url: process.env.NUXT_SITE_URL
  },
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
  modules: ['@nuxtjs/tailwindcss', '@nuxt/content', '@nuxtjs/robots', '@nuxtjs/sitemap'],
  robots: {
    groups: [
      {
        userAgent: '*',
        allow: '/'
      }
    ]
  },
  css: ['~/assets/css/main.css'],
  nitro: {
    preset: 'static',
    output: {
      publicDir: 'dist',
    },
  },
  routeRules: {
    '/**': { prerender: true },
  },
})
