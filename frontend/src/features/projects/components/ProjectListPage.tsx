import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { AxiosError } from "axios";
import { FolderKanban, Plus } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { PageHeader } from "@/components/layout/PageHeader";
import { formatDate, slugify } from "@/lib/utils";
import { extractApiError } from "@/lib/errors";
import { useProjects, useCreateProject } from "../hooks/use-projects";

export function ProjectListPage() {
  const { t } = useTranslation();
  const { orgSlug } = useParams<{ orgSlug: string }>();
  const { data, isLoading } = useProjects(orgSlug!);
  const projects = data?.data ?? [];

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.15 }}
      className="space-y-6"
    >
      <PageHeader
        title={t("projects.title")}
        actions={<CreateProjectDialog orgSlug={orgSlug!} />}
      />

      {isLoading && (
        <div className="grid gap-3 sm:grid-cols-2">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-24 rounded-lg" />
          ))}
        </div>
      )}

      {!isLoading && projects.length === 0 && (
        <EmptyState
          icon={FolderKanban}
          message={t("projects.emptyState")}
          actions={<CreateProjectDialog orgSlug={orgSlug!} />}
        />
      )}

      {!isLoading && projects.length > 0 && (
        <div className="grid gap-3 sm:grid-cols-2">
          {projects.map((project, i) => (
            <motion.div
              key={project.id}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: i * 0.03 }}
            >
              <Link to={`/${orgSlug}/${project.slug}`}>
                <Card className="p-4 hover:bg-card/80 hover:-translate-y-0.5 transition-all duration-200 cursor-pointer group">
                  <div className="flex items-start gap-3">
                    <div className="size-9 rounded-md bg-primary/10 flex items-center justify-center shrink-0 group-hover:bg-primary/15 transition-colors">
                      <FolderKanban className="size-4 text-primary" />
                    </div>
                    <div className="min-w-0 flex-1">
                      <h3 className="font-medium text-sm truncate">
                        {project.name}
                      </h3>
                      <p className="text-xs text-muted-foreground mt-0.5">
                        {project.slug}
                      </p>
                      <p className="text-xs text-muted-foreground/60 mt-1">
                        {formatDate(project.created_at)}
                      </p>
                    </div>
                  </div>
                </Card>
              </Link>
            </motion.div>
          ))}
        </div>
      )}
    </motion.div>
  );
}

function CreateProjectDialog({ orgSlug }: { orgSlug: string }) {
  const { t } = useTranslation();
  const createProject = useCreateProject(orgSlug);
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugEdited, setSlugEdited] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleNameChange = (value: string) => {
    setName(value);
    if (!slugEdited) {
      setSlug(slugify(value));
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    createProject.mutate(
      { name, slug },
      {
        onSuccess: () => {
          setOpen(false);
          setName("");
          setSlug("");
          setSlugEdited(false);
        },
        onError: (err) => {
          if (err instanceof AxiosError && err.response?.status === 409) {
            setError(t("projects.slugTaken"));
          } else {
            setError(extractApiError(err, t("errors.somethingWentWrong")));
          }
        },
      },
    );
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm" className="gap-1.5">
          <Plus className="size-3.5" />
          {t("projects.create")}
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("projects.create")}</DialogTitle>
          <DialogDescription className="sr-only">
            {t("projects.create")}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2 text-sm text-destructive">
              {error}
            </div>
          )}
          <div className="space-y-2">
            <Label>{t("projects.name")}</Label>
            <Input
              value={name}
              onChange={(e) => handleNameChange(e.target.value)}
              required
              minLength={1}
              maxLength={255}
              autoFocus
            />
          </div>
          <div className="space-y-2">
            <Label>{t("projects.slug")}</Label>
            <Input
              value={slug}
              onChange={(e) => {
                setSlug(e.target.value);
                setSlugEdited(true);
              }}
              required
              minLength={2}
              maxLength={100}
              pattern="^[a-z0-9][a-z0-9-]*[a-z0-9]$"
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => setOpen(false)}
            >
              {t("common.cancel")}
            </Button>
            <Button type="submit" disabled={createProject.isPending}>
              {createProject.isPending
                ? t("common.loading")
                : t("common.create")}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
