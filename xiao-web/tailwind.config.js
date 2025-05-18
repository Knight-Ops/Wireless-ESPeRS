/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,svelte,js,ts}'],   // tells JIT where to look
  plugins: [require('daisyui')],                 // 🟢 add daisyUI
  daisyui: {
    themes: ['dark'],                   // pick or customise themes
    // optional: base: false, utils: false, logs: false
  }
};