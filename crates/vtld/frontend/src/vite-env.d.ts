/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module 'swagger-ui-dist' {
  export function SwaggerUIBundle(config: any): any
  export const SwaggerUIStandalonePreset: any
}
