import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        display: ['Rajdhani', 'Inter', 'system-ui', 'sans-serif'],
        body: ['Inter', 'system-ui', 'sans-serif'],
      },
      colors: {
        rust: {
          ember: '#ff6b35',
          steel: '#9fb3c8',
          dark: '#090b10',
          panel: '#121720',
          cyan: '#40e0ff',
        },
      },
      boxShadow: {
        ember: '0 0 38px rgba(255, 107, 53, 0.3)',
        cyan: '0 0 34px rgba(64, 224, 255, 0.22)',
      },
      animation: {
        drift: 'drift 18s ease-in-out infinite',
        pulseGlow: 'pulseGlow 3s ease-in-out infinite',
        scan: 'scan 7s linear infinite',
        rise: 'rise 0.8s ease-out both',
      },
      keyframes: {
        drift: {
          '0%, 100%': { transform: 'translate3d(0, 0, 0) rotate(0deg)' },
          '50%': { transform: 'translate3d(28px, -22px, 0) rotate(2deg)' },
        },
        pulseGlow: {
          '0%, 100%': { opacity: '0.45', transform: 'scale(1)' },
          '50%': { opacity: '0.85', transform: 'scale(1.04)' },
        },
        scan: {
          '0%': { transform: 'translateY(-100%)' },
          '100%': { transform: 'translateY(100%)' },
        },
        rise: {
          '0%': { opacity: '0', transform: 'translateY(18px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
    },
  },
  plugins: [],
} satisfies Config;
