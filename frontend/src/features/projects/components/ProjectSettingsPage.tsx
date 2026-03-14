import { useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { extractApiError } from "@/lib/errors";
import { MoreHorizontal, Plus, Trash2, UserMinus } from "lucide-react";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/layout/PageHeader";
import {
  useProject,
  useUpdateProject,
  useDeleteProject,
  useProjectMembers,
  useUpdateProjectMemberRole,
  useRemoveProjectMember,
  useAddProjectMember,
} from "../hooks/use-projects";
import type { ProjectRole } from "@/lib/api-types";

export function ProjectSettingsPage() {
  const { t } = useTranslation();
  const { orgSlug, projectSlug } = useParams<{
    orgSlug: string;
    projectSlug: string;
  }>();
  const { data: project, isLoading } = useProject(orgSlug!, projectSlug!);

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-64" />
      </div>
    );
  }

  if (!project) return null;

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.15 }}
      className="space-y-6"
    >
      <PageHeader title={`${project.name} — ${t("projects.settings")}`} />

      <Tabs defaultValue="general">
        <TabsList>
          <TabsTrigger value="general">{t("projects.settings")}</TabsTrigger>
          <TabsTrigger value="members">{t("projects.members")}</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="mt-4 space-y-4">
          <GeneralTab
            orgSlug={orgSlug!}
            projectSlug={projectSlug!}
            projectName={project.name}
          />
        </TabsContent>

        <TabsContent value="members" className="mt-4">
          <MembersTab orgSlug={orgSlug!} projectSlug={projectSlug!} />
        </TabsContent>
      </Tabs>
    </motion.div>
  );
}

function GeneralTab({
  orgSlug,
  projectSlug,
  projectName,
}: {
  orgSlug: string;
  projectSlug: string;
  projectName: string;
}) {
  const { t } = useTranslation();
  const confirm = useConfirm();
  const updateProject = useUpdateProject(orgSlug, projectSlug);
  const deleteProject = useDeleteProject(orgSlug, projectSlug);
  const [name, setName] = useState(projectName);
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("projects.name")}</CardTitle>
        </CardHeader>
        <CardContent>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              setError(null);
              updateProject.mutate(
                { name },
                {
                  onSuccess: () => toast.success(t("common.save")),
                  onError: (err) => {
                    setError(extractApiError(err, t("errors.somethingWentWrong")));
                  },
                },
              );
            }}
            className="flex gap-2"
          >
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              className="max-w-xs"
            />
            <Button type="submit" size="sm" disabled={updateProject.isPending}>
              {t("common.save")}
            </Button>
          </form>
          {error && (
            <p className="text-sm text-destructive mt-2">{error}</p>
          )}
        </CardContent>
      </Card>

      <Card className="border-destructive/30">
        <CardHeader>
          <CardTitle className="text-base text-destructive">
            {t("projects.delete")}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Button
            variant="destructive"
            size="sm"
            onClick={async () => {
              const ok = await confirm({ description: t("common.confirmDelete", { name: projectName }), destructive: true });
              if (!ok) return;
              deleteProject.mutate();
            }}
            disabled={deleteProject.isPending}
          >
            <Trash2 className="size-3.5 mr-1.5" />
            {t("projects.delete")}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}

function MembersTab({
  orgSlug,
  projectSlug,
}: {
  orgSlug: string;
  projectSlug: string;
}) {
  const { t } = useTranslation();
  const confirm = useConfirm();
  const { data, isLoading } = useProjectMembers(orgSlug, projectSlug);
  const updateRole = useUpdateProjectMemberRole(orgSlug, projectSlug);
  const removeMember = useRemoveProjectMember(orgSlug, projectSlug);
  const addMember = useAddProjectMember(orgSlug, projectSlug);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newUserId, setNewUserId] = useState("");
  const [newRole, setNewRole] = useState<"admin" | "editor" | "viewer">("viewer");

  const roleLabel = (role: ProjectRole) => t(`projects.role.${role}`);
  const allRoles: ProjectRole[] = ["admin", "editor", "viewer"];

  if (isLoading) {
    return <Skeleton className="h-48" />;
  }

  const members = data?.data ?? [];

  return (
    <div className="space-y-3">
      <div className="flex justify-end">
        {showAddForm ? (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              addMember.mutate(
                { user_id: newUserId, role: newRole },
                {
                  onSuccess: () => {
                    setShowAddForm(false);
                    setNewUserId("");
                    setNewRole("viewer");
                  },
                },
              );
            }}
            className="flex gap-2 items-end"
          >
            <Input
              value={newUserId}
              onChange={(e) => setNewUserId(e.target.value)}
              placeholder={t("projects.userId")}
              required
              className="max-w-[200px] font-mono text-xs"
            />
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size="sm">
                  {roleLabel(newRole)}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                {allRoles.map((r) => (
                  <DropdownMenuItem key={r} onClick={() => setNewRole(r)}>
                    {roleLabel(r)}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
            <Button type="submit" size="sm" disabled={addMember.isPending}>
              {t("common.create")}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setShowAddForm(false)}
            >
              {t("common.cancel")}
            </Button>
          </form>
        ) : (
          <Button size="sm" className="gap-1.5" onClick={() => setShowAddForm(true)}>
            <Plus className="size-3.5" />
            {t("projects.addMember")}
          </Button>
        )}
      </div>

    <Card>
      <CardContent className="p-0">
        {members.length === 0 ? (
          <div className="text-center py-8 text-sm text-muted-foreground">
            {t("common.noData")}
          </div>
        ) : (
          <div className="divide-y divide-border">
            {members.map((member) => (
              <div
                key={member.user_id}
                className="flex items-center gap-3 px-4 py-3 odd:bg-muted/30 hover:bg-muted/50 transition-colors"
              >
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium truncate font-mono">
                    {member.user_id.slice(0, 8)}
                  </p>
                </div>
                <Badge variant="secondary" className="text-xs shrink-0">
                  {roleLabel(member.role)}
                </Badge>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" size="icon-xs">
                      <MoreHorizontal className="size-3.5" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    {allRoles
                      .filter((r) => r !== member.role)
                      .map((r) => (
                        <DropdownMenuItem
                          key={r}
                          onClick={() =>
                            updateRole.mutate({
                              userId: member.user_id,
                              role: r,
                            })
                          }
                        >
                          {roleLabel(r)}
                        </DropdownMenuItem>
                      ))}
                    <DropdownMenuItem
                      onClick={async () => {
                        const ok = await confirm({ description: t("common.confirmRemoveMember"), destructive: true });
                        if (!ok) return;
                        removeMember.mutate(member.user_id);
                      }}
                      className="text-destructive focus:text-destructive"
                    >
                      <UserMinus className="size-3.5 mr-1.5" />
                      {t("common.delete")}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
    </div>
  );
}
