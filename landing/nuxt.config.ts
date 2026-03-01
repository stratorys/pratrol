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
      script: [
        {
          type: 'text/javascript',
          innerHTML: 'window.$crisp=[];window.CRISP_WEBSITE_ID="e64c14d7-fa4f-47b9-ac74-ff1520e2c842";(function(){var d=document;var s=d.createElement("script");s.src="https://client.crisp.chat/l.js";s.async=1;d.head.appendChild(s);})();'
        }
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
