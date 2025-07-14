import daisyui from 'daisyui'

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {},
  },
  plugins: [daisyui],
  daisyui: {
    themes: [
      {
        s2secretTheme: {
          "primary": "#ea580c",
          "secondary": "#fcd34d",
          "accent": "#fdba74",
          "neutral": "#f5f5f4",
          "base-100": "#1f2937",
          "info": "#757575",
          "success": "#00ff00",
          "warning": "#fb7185",
          "error": "#fd2c30",
        },
      }
    ]
  }
}
