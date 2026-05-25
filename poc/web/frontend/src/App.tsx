import { useState } from 'react';
import { Dashboard } from './components/Dashboard/Dashboard';
import { AuthModal } from './components/Landingpage/AuthModal';
import { FeaturesSection } from './components/Landingpage/FeaturesSection';
import { Footer } from './components/Landingpage/Footer';
import { HeroSection } from './components/Landingpage/HeroSection';
import { NavBar } from './components/Landingpage/NavBar';
import { NewsSection } from './components/Landingpage/NewsSection';

export default function App() {
  const [isAuthModalOpen, setAuthModalOpen] = useState(false);
  const [isDashboardOpen, setDashboardOpen] = useState(false);

  const handleAuthSuccess = () => {
    setAuthModalOpen(false);
    setDashboardOpen(true);
  };

  if (isDashboardOpen) {
    return <Dashboard onSignOut={() => setDashboardOpen(false)} />;
  }

  return (
    <div className="min-h-screen bg-neutral-950 font-sans selection:bg-amber-500 selection:text-neutral-950">
      <NavBar onOpenAuth={() => setAuthModalOpen(true)} />
      <HeroSection onOpenAuth={() => setAuthModalOpen(true)} />
      <FeaturesSection />
      <NewsSection />
      <Footer />
      <AuthModal
        isOpen={isAuthModalOpen}
        onClose={() => setAuthModalOpen(false)}
        onAuthSuccess={handleAuthSuccess}
      />
    </div>
  );
}
