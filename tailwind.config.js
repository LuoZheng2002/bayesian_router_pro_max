/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './index.html',
    './src/**/*.rs', // Tailwind will scan your Rust components for class names
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}

