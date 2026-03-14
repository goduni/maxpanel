import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { PageHeader } from "@/components/layout/PageHeader";
import { Inbox, Link as LinkIcon } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { formatDate } from "@/lib/utils";
import { useMyInvites } from "../hooks/use-organizations";

export function InvitesPage() {
  const { t } = useTranslation();
  const { data, isLoading } = useMyInvites();
  const invites = data?.data ?? [];

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.15 }}
      className="space-y-6"
    >
      <PageHeader title={t("invites.title")} />

      {isLoading && (
        <div className="space-y-3">
          {[1, 2].map((i) => (
            <Skeleton key={i} className="h-20 rounded-lg" />
          ))}
        </div>
      )}

      {!isLoading && invites.length === 0 && (
        <EmptyState
          icon={Inbox}
          message={t("invites.emptyState")}
        />
      )}

      {!isLoading && invites.length > 0 && (
        <div className="space-y-3">
          {invites.map((invite, i) => (
            <motion.div
              key={invite.id}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: i * 0.03 }}
            >
              <Card className="p-4">
                <div className="flex items-center gap-3">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <Badge variant="secondary" className="text-xs">
                        {t(`orgs.role.${invite.role}`)}
                      </Badge>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">
                      {t("invites.expiresAt")}:{" "}
                      {formatDate(invite.expires_at)}
                    </p>
                  </div>
                  <div className="flex items-center gap-1.5 text-xs text-muted-foreground shrink-0">
                    <LinkIcon className="size-3" />
                    {t("invites.accept")}
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
          <p className="text-xs text-muted-foreground text-center">
            {t("invites.acceptViaLink")}
          </p>
        </div>
      )}
    </motion.div>
  );
}
