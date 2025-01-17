const colors = require('tailwindcss/colors')
const defaultTheme = require('tailwindcss/defaultTheme')

module.exports = {
  content: [
    "index.html",
    "./src/**/*.rs"
  ],
  theme: {
    fontFamily: {
      sans: ['Proxima Nova', ...defaultTheme.fontFamily.sans],
      mono: [...defaultTheme.fontFamily.mono],
      serif: [...defaultTheme.fontFamily.serif]
    },
    extend: {
      // Our application colour palette is defined here.
      colors: {
        'base': colors.gray,
        'primary': colors.blue,
        'secondary': colors.yellow,
        'error': colors.red
      }
    }
  },
  plugins: [],
}
