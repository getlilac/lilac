import nextra from 'nextra'
 
const withNextra = nextra({
  search: true,
})
 
export default withNextra({
  output: 'export',
  turbopack: {
    resolveAlias: {
      'next-mdx-import-source-file': './src/mdx-components.tsx',
    },
  },
})