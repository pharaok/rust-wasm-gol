/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.rs"],
  theme: {
    extend: {
      fontFamily: {
        mono: ["Roboto One", "monospace"],
      },
      keyframes: {
        "slide-in-from-right": {
          "0%": {
            opacity: "0",
            transform: "translateX(100%)",
          },
          "100%": {
            opacity: "1",
            transform: "translateX(0)",
          },
        },
        "fade-out": {
          "0%": { opacity: "1" },
          "100%": { opacity: "0" },
        },
      },
      animation: {
        "slide-in-from-right": "slide-in-from-right 0.3s ease-in-out forwards",
        "fade-out": "fade-out 0.3s ease-in forwards",
      },
    },
  },
  plugins: [],
};
