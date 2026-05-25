import { useEffect, useState } from 'react';
import { FaBoltLightning, FaDiscord, FaGithub, FaGoogle } from 'react-icons/fa6';

import { CloseIcon } from './icons';

type AuthModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onAuthSuccess: () => void;
};

export const AuthModal = ({ isOpen, onClose, onAuthSuccess }: AuthModalProps) => {
  const [isLogin, setIsLogin] = useState(true);

  useEffect(() => {
    let resetTimer: number | undefined;

    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
      resetTimer = window.setTimeout(() => setIsLogin(true), 400);
    }

    return () => {
      document.body.style.overflow = '';
      window.clearTimeout(resetTimer);
    };
  }, [isOpen]);

  return (
    <div
      className={`fixed inset-0 z-[100] flex items-center justify-center p-4 transition-all duration-300 ${
        isOpen ? 'opacity-100 visible' : 'opacity-0 invisible pointer-events-none'
      }`}
    >
      <div className="absolute inset-0 bg-black/80" onClick={onClose} />

      <div
        className={`relative w-full max-w-[420px] overflow-hidden rounded-[20px] border border-[#242424] bg-[#141414] transition-all duration-500 ease-[cubic-bezier(0.16,1,0.3,1)] ${
          isOpen ? 'translate-y-0 scale-100 opacity-100' : 'translate-y-6 scale-95 opacity-0'
        }`}
      >
        <button
          onClick={onClose}
          className="absolute right-5 top-5 z-10 text-[#555] transition-colors hover:text-white"
          aria-label="Close authentication modal"
        >
          <CloseIcon />
        </button>

        <div className="px-9 pt-9 pb-0">
          <div className="flex items-center gap-2.5 mb-8">
            <div className="w-9 h-9 bg-amber-500 rounded-[9px] flex items-center justify-center">
              <FaBoltLightning className="w-[18px] h-[18px] text-[#0a0a0a]" />
            </div>
            <span className="text-[15px] font-extrabold text-white tracking-tight">Arena</span>
          </div>

          <div className="relative flex bg-[#1a1a1a] rounded-[10px] mb-7 overflow-hidden border border-[#242424]">
            <div
              className={`absolute top-0 left-0 h-full w-1/2 bg-amber-500 transition-transform duration-300 ease-[cubic-bezier(0.16,1,0.3,1)] ${
                isLogin ? 'translate-x-0' : 'translate-x-full'
              }`}
            />

            <button
              onClick={() => setIsLogin(true)}
              className={`relative z-10 flex-1 py-2.5 text-[11px] font-extrabold tracking-widest uppercase transition-colors duration-300 cursor-pointer border-none bg-transparent ${
                isLogin ? 'text-[#0a0a0a]' : 'text-[#666] hover:text-white'
              }`}
            >
              Sign in
            </button>
            <button
              onClick={() => setIsLogin(false)}
              className={`relative z-10 flex-1 py-2.5 text-[11px] font-extrabold tracking-widest uppercase transition-colors duration-300 cursor-pointer border-none bg-transparent ${
                !isLogin ? 'text-[#0a0a0a]' : 'text-[#666] hover:text-white'
              }`}
            >
              Register
            </button>
          </div>

          <h2 className="text-[26px] font-extrabold text-white tracking-[-0.04em] leading-tight mb-1.5">
            {isLogin ? 'Good to see you back.' : 'Create your account.'}
          </h2>
          <p className="text-[13px] text-[#4a4a4a] leading-relaxed mb-7">
            {isLogin ? 'Sign in to your account to continue.' : 'Set up your identity and start competing.'}
          </p>
        </div>

        <div className="px-9 pb-7 flex flex-col gap-3.5">
          <div
            style={{ transition: 'grid-template-rows 0.36s cubic-bezier(0.16,1,0.3,1), opacity 0.28s ease' }}
            className={`grid ${isLogin ? 'grid-rows-[0fr] opacity-0' : 'grid-rows-[1fr] opacity-100'}`}
          >
            <div className="overflow-hidden">
              <div className="flex flex-col gap-1.5 pb-3.5">
                <label className="text-[11px] font-bold tracking-[0.14em] uppercase text-[#444]">Callsign</label>
                <input
                  type="text"
                  placeholder="Your username"
                  className="w-full bg-[#0f0f0f] border border-[#242424] rounded-[10px] px-4 py-3.5 text-sm text-white outline-none placeholder:text-[#2e2e2e] focus:border-[#3a3a3a] focus:bg-[#111] transition-all"
                />
              </div>
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <label className="text-[11px] font-bold tracking-[0.14em] uppercase text-[#444]">Email</label>
            <input
              type="email"
              placeholder="you@example.com"
              className="w-full bg-[#0f0f0f] border border-[#242424] rounded-[10px] px-4 py-3.5 text-sm text-white outline-none placeholder:text-[#2e2e2e] focus:border-[#3a3a3a] focus:bg-[#111] transition-all"
            />
          </div>

          <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between">
              <label className="text-[11px] font-bold tracking-[0.14em] uppercase text-[#444]">Password</label>
              {isLogin && (
                <button className="text-[12px] text-[#444] hover:text-amber-500 transition-colors bg-transparent border-none cursor-pointer">
                  Forgot password?
                </button>
              )}
            </div>
            <input
              type="password"
              placeholder="••••••••••"
              className="w-full bg-[#0f0f0f] border border-[#242424] rounded-[10px] px-4 py-3.5 text-sm text-white outline-none placeholder:text-[#2e2e2e] focus:border-[#3a3a3a] focus:bg-[#111] transition-all"
            />
          </div>

          <button
            onClick={onAuthSuccess}
            className="mt-1 w-full bg-amber-500 hover:bg-amber-400 border-none rounded-[10px] py-[15px] text-[13px] font-extrabold tracking-[0.12em] uppercase text-[#0a0a0a] cursor-pointer transition-all hover:-translate-y-px active:translate-y-0"
          >
            {isLogin ? 'Sign in' : 'Create account'}
          </button>

          <div className="flex items-center gap-3 my-0.5">
            <div className="flex-1 h-px bg-[#1e1e1e]" />
            <span className="text-[11px] text-[#303030] font-semibold uppercase tracking-widest">or</span>
            <div className="flex-1 h-px bg-[#1e1e1e]" />
          </div>

          <div className="flex gap-2.5">
            {[
              {
                label: 'Google',
                icon: <FaGoogle className="w-4 h-4 text-[#4285F4]" />,
              },
              {
                label: 'Discord',
                icon: <FaDiscord className="w-4 h-4 text-[#5865F2]" />,
              },
              {
                label: 'GitHub',
                icon: <FaGithub className="w-4 h-4" />,
              },
            ].map(({ label, icon }) => (
              <button
                key={label}
                className="flex-1 flex items-center justify-center gap-2 bg-[#1a1a1a] border border-[#242424] hover:border-[#383838] rounded-[10px] py-3 text-[12px] font-semibold text-[#666] hover:text-[#ccc] transition-all cursor-pointer"
              >
                {icon}
                {label}
              </button>
            ))}
          </div>
        </div>

        <div className="px-9 py-[18px] border-t border-[#1a1a1a] bg-[#111] flex items-center justify-center gap-1.5">
          <span className="text-[12px] text-[#3a3a3a]">
            {isLogin ? 'No account yet?' : 'Already have one?'}
          </span>
          <button
            onClick={() => setIsLogin(!isLogin)}
            className="text-[12px] text-amber-500 font-bold bg-transparent border-none cursor-pointer"
          >
            {isLogin ? 'Create one' : 'Sign in'}
          </button>
        </div>
      </div>
    </div>
  );
};
