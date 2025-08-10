import React from 'react'
import { useThemeConfig } from 'nextra-theme-docs'

const config = {
  useNextSeoProps() {
    return {
      titleTemplate: '%s – Lilac'
    }
  },
  logo: <img src="/Lilac_8.png" alt="Lilac" />,
  project: {
    link: 'https://discord.gg/getlilac',
  },
  docsRepositoryBase: 'https://github.com/getlilac/lilac/tree/main/docs',
  footer: {
    text: 'Lilac Documentation',
  },
}

export default config