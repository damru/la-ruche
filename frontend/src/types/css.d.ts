declare module '*.css' {
  const content: string
  export default content
}

declare module '*.svg' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent
  export default component
}
