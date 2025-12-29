import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Turbo CDN',
  description: 'Next-generation intelligent download accelerator with automatic CDN optimization',
  base: '/turbo-cdn/',
  
  head: [
    ['link', { rel: 'icon', href: '/turbo-cdn/favicon.ico' }],
    ['meta', { name: 'theme-color', content: '#3eaf7c' }],
    ['meta', { name: 'og:type', content: 'website' }],
    ['meta', { name: 'og:title', content: 'Turbo CDN' }],
    ['meta', { name: 'og:description', content: 'Next-generation intelligent download accelerator' }],
  ],

  locales: {
    root: {
      label: 'English',
      lang: 'en',
    },
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh/',
      themeConfig: {
        nav: [
          { text: '首页', link: '/zh/' },
          { text: '指南', link: '/zh/guide/' },
          { text: 'API', link: '/zh/api/' },
          { text: '更新日志', link: 'https://github.com/loonghao/turbo-cdn/blob/main/CHANGELOG.md' },
        ],
        sidebar: {
          '/zh/guide/': [
            {
              text: '入门',
              items: [
                { text: '简介', link: '/zh/guide/' },
                { text: '快速开始', link: '/zh/guide/getting-started' },
                { text: '安装', link: '/zh/guide/installation' },
              ]
            },
            {
              text: '核心功能',
              items: [
                { text: '智能地理检测', link: '/zh/guide/geo-detection' },
                { text: 'CDN 质量评估', link: '/zh/guide/cdn-quality' },
                { text: '智能下载', link: '/zh/guide/smart-download' },
              ]
            },
            {
              text: '高级功能',
              items: [
                { text: '自适应并发', link: '/zh/guide/adaptive-concurrency' },
                { text: '智能分块', link: '/zh/guide/smart-chunking' },
                { text: 'DNS 缓存', link: '/zh/guide/dns-cache' },
              ]
            }
          ],
          '/zh/api/': [
            {
              text: 'API 参考',
              items: [
                { text: '概览', link: '/zh/api/' },
                { text: 'TurboCdn', link: '/zh/api/turbo-cdn' },
                { text: 'DownloadOptions', link: '/zh/api/download-options' },
              ]
            }
          ]
        },
        outline: {
          label: '页面导航'
        },
        docFooter: {
          prev: '上一页',
          next: '下一页'
        },
        lastUpdated: {
          text: '最后更新于'
        },
        editLink: {
          pattern: 'https://github.com/loonghao/turbo-cdn/edit/main/docs-site/:path',
          text: '在 GitHub 上编辑此页'
        }
      }
    }
  },

  themeConfig: {
    logo: '/logo.svg',
    
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/' },
      { text: 'API', link: '/api/' },
      { text: 'Changelog', link: 'https://github.com/loonghao/turbo-cdn/blob/main/CHANGELOG.md' },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction', link: '/guide/' },
            { text: 'Quick Start', link: '/guide/getting-started' },
            { text: 'Installation', link: '/guide/installation' },
          ]
        },
        {
          text: 'Core Features',
          items: [
            { text: 'Geographic Detection', link: '/guide/geo-detection' },
            { text: 'CDN Quality Assessment', link: '/guide/cdn-quality' },
            { text: 'Smart Download', link: '/guide/smart-download' },
          ]
        },
        {
          text: 'Advanced Features',
          items: [
            { text: 'Adaptive Concurrency', link: '/guide/adaptive-concurrency' },
            { text: 'Smart Chunking', link: '/guide/smart-chunking' },
            { text: 'DNS Cache', link: '/guide/dns-cache' },
            { text: 'Self Update', link: '/guide/self-update' },
          ]
        }
      ],
      '/api/': [
        {
          text: 'API Reference',
          items: [
            { text: 'Overview', link: '/api/' },
            { text: 'TurboCdn', link: '/api/turbo-cdn' },
            { text: 'DownloadOptions', link: '/api/download-options' },
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/loonghao/turbo-cdn' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2025 Hal'
    },

    search: {
      provider: 'local'
    },

    editLink: {
      pattern: 'https://github.com/loonghao/turbo-cdn/edit/main/docs-site/:path',
      text: 'Edit this page on GitHub'
    },

    lastUpdated: {
      text: 'Last updated'
    }
  }
})
