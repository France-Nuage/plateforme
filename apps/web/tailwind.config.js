const colors = require("tailwindcss/colors");

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./components/**/*.{js,vue,ts}",
    "./layouts/**/*.vue",
    "./pages/**/*.vue",
    "./plugins/**/*.{js,ts}",
    "./app.vue",
    "./error.vue",
  ],
  theme: {
    colors: {
      ...colors,
      ...{
        primary: {
          DEFAULT: "#1A4CE5",
        },
      },
    },
    extend: {},
  },
  plugins: [require("@tailwindcss/forms")],
};
