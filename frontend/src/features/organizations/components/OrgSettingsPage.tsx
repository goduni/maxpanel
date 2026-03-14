import { useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useMutation } from "@tanstack/react-query";
import { motion } from "motion/react";
import { extractApiError } from "@/lib/errors";
import {
  ArrowRightLeft,
  Copy,
  Crown,
  MoreHorizontal,
  Plus,
  Shield,
  Trash2,
  User,
  UserMinus,
} from "lucide-react";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/layout/PageHeader";
import {
  useOrganization,
  useUpdateOrg,
  useDeleteOrg,
  useOrgMembers,
  useUpdateMemberRole,
  useRemoveMember,
  useOrgInvites,
  useCreateInvite,
  useRevokeInvite,
} from "../hooks/use-organizations";
import { formatDate } from "@/lib/utils";
import * as orgApi from "../api";
import type { OrgRole, OrganizationMember } from "@/lib/api-types";

const ROLE_ICONS: Record<OrgRole, React.ElementType> = {
  owner: Crown,
  admin: Shield,
  member: User,
};

const ROLE_COLORS: Record<OrgRole, string> = {
  owner: "bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20",
  admin: "bg-primary/10 text-primary border-primary/20",
  member: "bg-muted text-muted-foreground border-border",
};

export function OrgSettingsPage() {
  const { t } = useTranslation();
  const { orgSlug } = useParams<{ orgSlug: string }>();
  const { data: org, isLoading } = useOrganization(orgSlug!);

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-10 w-64" />
        <Skeleton className="h-64" />
      </div>
    );
  }

  if (!org) return null;

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.15 }}
      className="space-y-6"
    >
      <PageHeader title={`${org.name} — ${t("orgs.settings")}`} />

      <Tabs defaultValue="general">
        <TabsList>
          <TabsTrigger value="general">{t("orgs.general")}</TabsTrigger>
          <TabsTrigger value="members">{t("orgs.members")}</TabsTrigger>
          <TabsTrigger value="invites">{t("orgs.invites")}</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="mt-6 space-y-6">
          <GeneralTab orgSlug={orgSlug!} orgName={org.name} />
        </TabsContent>

        <TabsContent value="members" className="mt-6">
          <MembersTab orgSlug={orgSlug!} />
        </TabsContent>

        <TabsContent value="invites" className="mt-6">
          <InvitesTab orgSlug={orgSlug!} />
        </TabsContent>
      </Tabs>
    </motion.div>
  );
}

function GeneralTab({
  orgSlug,
  orgName,
}: {
  orgSlug: string;
  orgName: string;
}) {
  const { t } = useTranslation();
  const confirm = useConfirm();
  const updateOrg = useUpdateOrg(orgSlug);
  const deleteOrg = useDeleteOrg(orgSlug);
  const [name, setName] = useState(orgName);
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="space-y-6">
      {/* Name */}
      <div className="space-y-2">
        <Label htmlFor="org-name" className="text-sm font-medium">
          {t("orgs.name")}
        </Label>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setError(null);
            updateOrg.mutate(
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
            id="org-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            minLength={1}
            maxLength={255}
            className="max-w-sm"
          />
          <Button type="submit" size="sm" disabled={updateOrg.isPending}>
            {t("common.save")}
          </Button>
        </form>
        {error && (
          <p className="text-sm text-destructive">{error}</p>
        )}
      </div>

      <Separator />

      {/* Transfer ownership */}
      <TransferOwnershipSection orgSlug={orgSlug} />

      <Separator />

      {/* Danger zone */}
      <div className="space-y-3">
        <div>
          <h3 className="text-sm font-medium text-destructive">
            {t("orgs.delete")}
          </h3>
          <p className="text-xs text-muted-foreground mt-0.5">
            {t("common.confirmDelete", { name: orgName })}
          </p>
        </div>
        <Button
          variant="destructive"
          size="sm"
          onClick={async () => {
            const ok = await confirm({
              description: t("common.confirmDelete", { name: orgName }),
              destructive: true,
            });
            if (!ok) return;
            deleteOrg.mutate();
          }}
          disabled={deleteOrg.isPending}
        >
          <Trash2 className="size-3.5 mr-1.5" />
          {t("orgs.delete")}
        </Button>
      </div>
    </div>
  );
}

function TransferOwnershipSection({ orgSlug }: { orgSlug: string }) {
  const { t } = useTranslation();
  const confirm = useConfirm();
  const { data: membersData } = useOrgMembers(orgSlug);
  const [selectedMemberId, setSelectedMemberId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const members = membersData?.data ?? [];
  // Only non-owner members can become the new owner
  const transferable = members.filter((m) => m.role !== "owner");
  const selectedMember = transferable.find((m) => m.user_id === selectedMemberId);

  const transfer = useMutation({
    mutationFn: () =>
      orgApi.transferOwnership(orgSlug, {
        new_owner_id: selectedMemberId!,
      }),
    onSuccess: () => {
      toast.success(t("common.save"));
      setSelectedMemberId(null);
    },
    onError: (err: unknown) => {
      setError(extractApiError(err, t("errors.somethingWentWrong")));
    },
  });

  return (
    <div className="space-y-3">
      <div>
        <h3 className="text-sm font-medium">{t("orgs.transferOwnership")}</h3>
        <p className="text-xs text-muted-foreground mt-0.5">
          {t("orgs.transferTo")}
        </p>
      </div>

      <div className="flex gap-2 items-center">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm" className="min-w-[200px] justify-start gap-2">
              {selectedMember ? (
                <>
                  <Avatar className="size-5">
                    <AvatarFallback className="text-[10px] bg-primary/10 text-primary">
                      {(selectedMember.user_name ?? "?")[0]?.toUpperCase()}
                    </AvatarFallback>
                  </Avatar>
                  <span className="truncate">
                    {selectedMember.user_name ?? selectedMember.user_email ?? selectedMember.user_id.slice(0, 8)}
                  </span>
                </>
              ) : (
                <span className="text-muted-foreground">{t("orgs.transferTo")}</span>
              )}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" className="w-64">
            {transferable.length === 0 ? (
              <div className="px-2 py-3 text-sm text-muted-foreground text-center">
                {t("common.noData")}
              </div>
            ) : (
              transferable.map((member) => (
                <DropdownMenuItem
                  key={member.user_id}
                  onClick={() => setSelectedMemberId(member.user_id)}
                  className="gap-2"
                >
                  <Avatar className="size-6">
                    <AvatarFallback className="text-[10px] bg-muted">
                      {(member.user_name ?? "?")[0]?.toUpperCase()}
                    </AvatarFallback>
                  </Avatar>
                  <div className="min-w-0">
                    <p className="text-sm truncate">
                      {member.user_name ?? member.user_id.slice(0, 8)}
                    </p>
                    {member.user_email && (
                      <p className="text-xs text-muted-foreground truncate">
                        {member.user_email}
                      </p>
                    )}
                  </div>
                  <Badge
                    variant="outline"
                    className={`text-[10px] ml-auto shrink-0 ${ROLE_COLORS[member.role]}`}
                  >
                    {t(`orgs.role.${member.role}`)}
                  </Badge>
                </DropdownMenuItem>
              ))
            )}
          </DropdownMenuContent>
        </DropdownMenu>

        <Button
          size="sm"
          disabled={!selectedMemberId || transfer.isPending}
          onClick={async () => {
            const name = selectedMember?.user_name ?? selectedMember?.user_email ?? selectedMember?.user_id.slice(0, 8);
            const ok = await confirm({
              description: `${t("orgs.transferOwnership")}: ${name}?`,
              destructive: true,
            });
            if (!ok) return;
            setError(null);
            transfer.mutate();
          }}
          className="gap-1.5"
        >
          <ArrowRightLeft className="size-3.5" />
          {t("orgs.transferOwnership")}
        </Button>
      </div>
      {error && <p className="text-sm text-destructive">{error}</p>}
    </div>
  );
}

function MembersTab({ orgSlug }: { orgSlug: string }) {
  const { t } = useTranslation();
  const confirm = useConfirm();
  const { data, isLoading } = useOrgMembers(orgSlug);
  const updateRole = useUpdateMemberRole(orgSlug);
  const removeMember = useRemoveMember(orgSlug);

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[1, 2, 3].map((i) => (
          <Skeleton key={i} className="h-16 rounded-lg" />
        ))}
      </div>
    );
  }

  const members = data?.data ?? [];

  return (
    <div className="space-y-3">
      {members.map((member) => (
        <MemberRow
          key={member.user_id}
          member={member}
          onUpdateRole={(role) =>
            updateRole.mutate({ userId: member.user_id, role })
          }
          onRemove={async () => {
            const ok = await confirm({
              description: t("common.confirmRemoveMember"),
              destructive: true,
            });
            if (!ok) return;
            removeMember.mutate(member.user_id);
          }}
        />
      ))}
    </div>
  );
}

function MemberRow({
  member,
  onUpdateRole,
  onRemove,
}: {
  member: OrganizationMember;
  onUpdateRole: (role: OrgRole) => void;
  onRemove: () => void;
}) {
  const { t } = useTranslation();
  const RoleIcon = ROLE_ICONS[member.role];
  const initials = (member.user_name ?? "?")[0]?.toUpperCase() ?? "?";

  return (
    <div className="flex items-center gap-3 px-4 py-3 rounded-lg border border-border/50 hover:border-border hover:bg-muted/30 transition-all">
      <Avatar className="size-9">
        <AvatarFallback className="text-xs bg-primary/10 text-primary font-medium">
          {initials}
        </AvatarFallback>
      </Avatar>

      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium truncate">
          {member.user_name ?? member.user_id.slice(0, 8)}
        </p>
        {member.user_email && (
          <p className="text-xs text-muted-foreground truncate">
            {member.user_email}
          </p>
        )}
      </div>

      <Badge
        variant="outline"
        className={`text-xs shrink-0 gap-1 ${ROLE_COLORS[member.role]}`}
      >
        <RoleIcon className="size-3" />
        {t(`orgs.role.${member.role}`)}
      </Badge>

      {member.role !== "owner" && (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon-xs" aria-label={t("common.moreOptions")}>
              <MoreHorizontal className="size-3.5" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            {member.role !== "admin" && (
              <DropdownMenuItem onClick={() => onUpdateRole("admin")}>
                <Shield className="size-3.5 mr-1.5" />
                {t("orgs.makeAdmin")}
              </DropdownMenuItem>
            )}
            {member.role !== "member" && (
              <DropdownMenuItem onClick={() => onUpdateRole("member")}>
                <User className="size-3.5 mr-1.5" />
                {t("orgs.makeMember")}
              </DropdownMenuItem>
            )}
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={onRemove}
              className="text-destructive focus:text-destructive"
            >
              <UserMinus className="size-3.5 mr-1.5" />
              {t("orgs.removeMember")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )}
    </div>
  );
}

function InvitesTab({ orgSlug }: { orgSlug: string }) {
  const { t } = useTranslation();
  const confirm = useConfirm();
  const { data, isLoading } = useOrgInvites(orgSlug);
  const revokeInvite = useRevokeInvite(orgSlug);
  const [dialogOpen, setDialogOpen] = useState(false);

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[1, 2].map((i) => (
          <Skeleton key={i} className="h-16 rounded-lg" />
        ))}
      </div>
    );
  }

  const invites = data?.data ?? [];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          {invites.length > 0
            ? `${invites.length} ${t("invites.title").toLowerCase()}`
            : t("invites.emptyState")}
        </p>
        <CreateInviteDialog
          orgSlug={orgSlug}
          open={dialogOpen}
          onOpenChange={setDialogOpen}
        />
      </div>

      {invites.length > 0 && (
        <div className="space-y-2">
          {invites.map((invite) => (
            <div
              key={invite.id}
              className="flex items-center gap-3 px-4 py-3 rounded-lg border border-border/50 hover:border-border hover:bg-muted/30 transition-all"
            >
              <div className="size-9 rounded-full bg-muted flex items-center justify-center shrink-0">
                <span className="text-xs font-medium text-muted-foreground">
                  {invite.email[0]?.toUpperCase()}
                </span>
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium truncate">{invite.email}</p>
                <p className="text-xs text-muted-foreground">
                  {t("invites.expiresAt")}: {formatDate(invite.expires_at)}
                </p>
              </div>
              <Badge
                variant="outline"
                className={`text-xs shrink-0 ${ROLE_COLORS[invite.role]}`}
              >
                {t(`orgs.role.${invite.role}`)}
              </Badge>
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={async () => {
                  const ok = await confirm({
                    description: t("common.confirmRevokeInvite"),
                    destructive: true,
                  });
                  if (!ok) return;
                  revokeInvite.mutate(invite.id);
                }}
              >
                <Trash2 className="size-3.5 text-destructive" />
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function CreateInviteDialog({
  orgSlug,
  open,
  onOpenChange,
}: {
  orgSlug: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { t } = useTranslation();
  const createInvite = useCreateInvite(orgSlug);
  const [email, setEmail] = useState("");
  const [role, setRole] = useState<"admin" | "member">("member");
  const [inviteToken, setInviteToken] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    createInvite.mutate(
      { email, role },
      {
        onSuccess: (data) => {
          setInviteToken(data.token);
        },
        onError: (err) => {
          setError(extractApiError(err, t("errors.somethingWentWrong")));
        },
      },
    );
  };

  const inviteUrl = inviteToken
    ? `${window.location.origin}/invite/${inviteToken}`
    : null;

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        onOpenChange(v);
        if (!v) {
          setEmail("");
          setRole("member");
          setInviteToken(null);
          setError(null);
        }
      }}
    >
      <DialogTrigger asChild>
        <Button size="sm" className="gap-1.5">
          <Plus className="size-3.5" />
          {t("invites.createInvite")}
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("invites.createInvite")}</DialogTitle>
          <DialogDescription className="sr-only">
            {t("invites.createInvite")}
          </DialogDescription>
        </DialogHeader>

        {inviteUrl ? (
          <div className="space-y-3">
            <p className="text-sm text-muted-foreground">
              {t("common.sendInviteLink")}
            </p>
            <div className="flex gap-2">
              <Input
                value={inviteUrl}
                readOnly
                className="font-mono text-xs"
              />
              <Button
                variant="outline"
                size="icon"
                onClick={() => {
                  navigator.clipboard.writeText(inviteUrl);
                  toast.success(t("common.copied"));
                }}
              >
                <Copy className="size-3.5" />
              </Button>
            </div>
            <Button
              variant="outline"
              className="w-full"
              onClick={() => onOpenChange(false)}
            >
              {t("common.close")}
            </Button>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2 text-sm text-destructive">
                {error}
              </div>
            )}
            <div className="space-y-2">
              <Label>{t("auth.email")}</Label>
              <Input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                autoFocus
              />
            </div>
            <div className="space-y-2">
              <Label>{t("invites.role")}</Label>
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant={role === "member" ? "default" : "outline"}
                  size="sm"
                  onClick={() => setRole("member")}
                  className="gap-1.5"
                >
                  <User className="size-3.5" />
                  {t("orgs.role.member")}
                </Button>
                <Button
                  type="button"
                  variant={role === "admin" ? "default" : "outline"}
                  size="sm"
                  onClick={() => setRole("admin")}
                  className="gap-1.5"
                >
                  <Shield className="size-3.5" />
                  {t("orgs.role.admin")}
                </Button>
              </div>
            </div>
            <div className="flex justify-end gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
              >
                {t("common.cancel")}
              </Button>
              <Button type="submit" disabled={createInvite.isPending}>
                {createInvite.isPending
                  ? t("common.loading")
                  : t("invites.createInvite")}
              </Button>
            </div>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
