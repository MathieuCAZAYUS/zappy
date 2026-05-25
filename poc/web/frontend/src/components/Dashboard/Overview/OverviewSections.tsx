import { FeaturedStoreSection } from './FeaturedStoreSection';
import { HeroOverviewSection } from './HeroOverviewSection';
import { LoadoutSection } from './LoadoutSection';
import { RecentOperationsSection } from './RecentOperationsSection';
import { StatsOverviewSection } from './StatsOverviewSection';

export function OverviewSections() {
  return (
    <>
      <HeroOverviewSection />
      <StatsOverviewSection />

      <section className="grid grid-cols-1 lg:grid-cols-[0.95fr_1.05fr] items-start gap-7">
        <div>
          <LoadoutSection />
        </div>

        <div className="space-y-7">
          <RecentOperationsSection />
          <FeaturedStoreSection />
        </div>
      </section>
    </>
  );
}
