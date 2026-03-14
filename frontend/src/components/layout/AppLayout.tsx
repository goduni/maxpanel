import { useEffect } from "react";
import { Outlet, useParams } from "react-router-dom";
import { useIsMobile, useIsTablet } from "@/hooks/use-media-query";
import { useSidebarStore } from "@/stores/sidebar";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";
import { MobileHeader, BotBottomTabs } from "./MobileNav";

export function AppLayout() {
  const isMobile = useIsMobile();
  const isTablet = useIsTablet();

  // Auto-collapse sidebar on tablet
  const { setCollapsed } = useSidebarStore();
  useEffect(() => {
    if (isTablet) setCollapsed(true);
  }, [isTablet, setCollapsed]);

  const params = useParams();
  const hasBotTabs = isMobile && !!params.botId;

  if (isMobile) {
    return (
      <div className="min-h-dvh bg-background flex flex-col safe-top">
        <MobileHeader />
        <main className="flex-1 flex flex-col overflow-hidden">
          <div className={`mx-auto max-w-7xl p-4 ${hasBotTabs ? "pb-20" : "pb-4"} w-full flex-1 flex flex-col min-h-0 overflow-y-auto safe-bottom`}>
            <Outlet />
          </div>
        </main>
        <BotBottomTabs />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background flex">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <Header />
        <main className="flex-1 flex flex-col overflow-hidden">
          <div className="mx-auto max-w-7xl p-6 w-full flex-1 flex flex-col min-h-0 overflow-y-auto">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
