import { FaArrowRightLong } from 'react-icons/fa6';

import { CARD_THEMES, MOCK_FEATURES } from './mockData';

export const FeaturesSection = () => {
  const loopingFeatures = [...MOCK_FEATURES, ...MOCK_FEATURES];

  return (
    <section className="py-32 bg-neutral-950 border-t border-neutral-900 overflow-hidden">
      <div className="max-w-7xl mx-auto px-6">
        <div className="mb-12 flex flex-col md:flex-row md:items-end justify-between gap-6">
          <div>
            <h2 className="text-3xl md:text-4xl font-black text-white uppercase tracking-tighter mb-4">Core Architecture</h2>
            <div className="w-12 h-1 bg-amber-500"></div>
            <p className="mt-6 text-neutral-400 font-medium max-w-xl leading-relaxed">
              Built from the ground up for stability, speed, and fairness. No bloat, just the essential tools for a massive combat experience.
            </p>
          </div>
          <div className="hidden md:flex items-center gap-2 text-neutral-500 uppercase tracking-widest text-[10px] font-bold">
            Live module stream
            <FaArrowRightLong className="w-4 h-4 animate-pulse" />
          </div>
        </div>

        <div className="-mx-6 overflow-hidden px-6 py-8 md:-mx-12 md:px-12">
          <div className="feature-marquee flex w-max gap-6">
            {loopingFeatures.map((feature, index) => {
              const theme = CARD_THEMES[index % CARD_THEMES.length];

              return (
                <div
                  key={`${feature.id}-${index}`}
                  className={`flex-shrink-0 w-[85vw] sm:w-[400px] bg-neutral-900 rounded-2xl border border-neutral-800 p-8 cursor-pointer group transition-all duration-500 ${theme.border} ${theme.shadow} flex flex-col justify-between`}
                >
                  <div>
                    <div className={`w-16 h-16 rounded-xl flex items-center justify-center border border-neutral-800 transition-colors duration-500 mb-8 ${theme.text} ${theme.iconBg} ${theme.iconBorder}`}>
                      {feature.icon}
                    </div>
                    <h3 className="text-xl font-bold text-white uppercase tracking-wide mb-4">
                      {feature.title}
                    </h3>
                    <p className="text-neutral-400 leading-relaxed font-medium">
                      {feature.description}
                    </p>
                  </div>

                  <div className={`mt-8 text-xs font-bold uppercase tracking-widest opacity-0 group-hover:opacity-100 transition-opacity duration-300 ${theme.text}`}>
                    Initialize Module &rarr;
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </section>
  );
};
