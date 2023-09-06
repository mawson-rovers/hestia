/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./**/*.{html,js}"],
  theme: {
    fontFamily: {
      sans: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Ubuntu', 'Droid Sans', 'Helvetica Neue', 'sans-serif'],
      serif: ['serif'],
    },
    extend: {
      colors: {
        'uts-blue': '#0f4beb',
      },
    },
  },
  plugins: [],
}

