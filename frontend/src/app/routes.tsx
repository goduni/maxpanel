/* eslint-disable react-refresh/only-export-components */
import { lazy, Suspense } from "react";
import { createBrowserRouter } from "react-router-dom";
import { ProtectedRoute } from "@/components/layout/ProtectedRoute";
import { GuestRoute } from "@/components/layout/GuestRoute";
import { AppLayout } from "@/components/layout/AppLayout";
import { NotFoundPage } from "@/components/layout/NotFoundPage";
import { ErrorBoundary } from "@/components/layout/ErrorBoundary";
import { Skeleton } from "@/components/ui/skeleton";
import { queryClient } from "@/app/providers";

// Lazy-loaded pages
const LoginPage = lazy(() =>
  import("@/features/auth/components/LoginPage").then((m) => ({
    default: m.LoginPage,
  })),
);
const RegisterPage = lazy(() =>
  import("@/features/auth/components/RegisterPage").then((m) => ({
    default: m.RegisterPage,
  })),
);
const SettingsPage = lazy(() =>
  import("@/features/auth/components/SettingsPage").then((m) => ({
    default: m.SettingsPage,
  })),
);
const OrgListPage = lazy(() =>
  import("@/features/organizations/components/OrgListPage").then((m) => ({
    default: m.OrgListPage,
  })),
);
const OrgSettingsPage = lazy(() =>
  import("@/features/organizations/components/OrgSettingsPage").then((m) => ({
    default: m.OrgSettingsPage,
  })),
);
const InvitesPage = lazy(() =>
  import("@/features/organizations/components/InvitesPage").then((m) => ({
    default: m.InvitesPage,
  })),
);
const AcceptInvitePage = lazy(() =>
  import("@/features/organizations/components/AcceptInvitePage").then((m) => ({
    default: m.AcceptInvitePage,
  })),
);
const ProjectListPage = lazy(() =>
  import("@/features/projects/components/ProjectListPage").then((m) => ({
    default: m.ProjectListPage,
  })),
);
const ProjectSettingsPage = lazy(() =>
  import("@/features/projects/components/ProjectSettingsPage").then((m) => ({
    default: m.ProjectSettingsPage,
  })),
);
const BotListPage = lazy(() =>
  import("@/features/bots/components/BotListPage").then((m) => ({
    default: m.BotListPage,
  })),
);
const BotLayout = lazy(() =>
  import("@/features/bots/components/BotLayout").then((m) => ({
    default: m.BotLayout,
  })),
);
const BotOverviewPage = lazy(() =>
  import("@/features/bots/components/BotOverviewPage").then((m) => ({
    default: m.BotOverviewPage,
  })),
);
const BotSettingsPage = lazy(() =>
  import("@/features/bots/components/BotSettingsPage").then((m) => ({
    default: m.BotSettingsPage,
  })),
);
const EventsPage = lazy(() =>
  import("@/features/events/components/EventsPage").then((m) => ({
    default: m.EventsPage,
  })),
);
const ChatsPage = lazy(() =>
  import("@/features/chats/components/ChatsPage").then((m) => ({
    default: m.ChatsPage,
  })),
);
const ApiConsolePage = lazy(() =>
  import("@/features/api-console/components/ApiConsolePage").then((m) => ({
    default: m.ApiConsolePage,
  })),
);

function PageLoader() {
  return (
    <div className="p-6 space-y-4">
      <Skeleton className="h-8 w-48" />
      <Skeleton className="h-64" />
    </div>
  );
}

function SuspenseWrap({ children }: { children: React.ReactNode }) {
  return (
    <ErrorBoundary
      variant="route"
      onReset={() => queryClient.invalidateQueries()}
    >
      <Suspense fallback={<PageLoader />}>{children}</Suspense>
    </ErrorBoundary>
  );
}

export const router = createBrowserRouter([
  // Guest routes
  {
    path: "/login",
    element: (
      <GuestRoute>
        <SuspenseWrap>
          <LoginPage />
        </SuspenseWrap>
      </GuestRoute>
    ),
  },
  {
    path: "/register",
    element: (
      <GuestRoute>
        <SuspenseWrap>
          <RegisterPage />
        </SuspenseWrap>
      </GuestRoute>
    ),
  },

  // Invite acceptance
  {
    path: "/invite/:token",
    element: (
      <SuspenseWrap>
        <AcceptInvitePage />
      </SuspenseWrap>
    ),
  },

  // Protected app routes
  {
    element: (
      <ProtectedRoute>
        <AppLayout />
      </ProtectedRoute>
    ),
    children: [
      {
        index: true,
        element: (
          <SuspenseWrap>
            <OrgListPage />
          </SuspenseWrap>
        ),
      },
      {
        path: "invites",
        element: (
          <SuspenseWrap>
            <InvitesPage />
          </SuspenseWrap>
        ),
      },
      {
        path: "settings",
        element: (
          <SuspenseWrap>
            <SettingsPage />
          </SuspenseWrap>
        ),
      },

      // Org routes
      {
        path: ":orgSlug",
        children: [
          {
            index: true,
            element: (
              <SuspenseWrap>
                <ProjectListPage />
              </SuspenseWrap>
            ),
          },
          {
            path: "settings",
            element: (
              <SuspenseWrap>
                <OrgSettingsPage />
              </SuspenseWrap>
            ),
          },

          // Project routes
          {
            path: ":projectSlug",
            children: [
              {
                index: true,
                element: (
                  <SuspenseWrap>
                    <BotListPage />
                  </SuspenseWrap>
                ),
              },
              {
                path: "settings",
                element: (
                  <SuspenseWrap>
                    <ProjectSettingsPage />
                  </SuspenseWrap>
                ),
              },

              // Bot routes with layout
              {
                path: "bots/:botId",
                element: (
                  <SuspenseWrap>
                    <BotLayout />
                  </SuspenseWrap>
                ),
                children: [
                  {
                    index: true,
                    element: (
                      <SuspenseWrap>
                        <BotOverviewPage />
                      </SuspenseWrap>
                    ),
                  },
                  {
                    path: "chats",
                    element: (
                      <SuspenseWrap>
                        <ChatsPage />
                      </SuspenseWrap>
                    ),
                  },
                  {
                    path: "events",
                    element: (
                      <SuspenseWrap>
                        <EventsPage />
                      </SuspenseWrap>
                    ),
                  },
                  {
                    path: "console",
                    element: (
                      <SuspenseWrap>
                        <ApiConsolePage />
                      </SuspenseWrap>
                    ),
                  },
                  {
                    path: "settings",
                    element: (
                      <SuspenseWrap>
                        <BotSettingsPage />
                      </SuspenseWrap>
                    ),
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  },

  // 404
  { path: "*", element: <NotFoundPage /> },
]);
