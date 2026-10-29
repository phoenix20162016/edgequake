"use client";

/**
 * SPEC-179 — one refresh control, used wherever a model list is shown.
 *
 * Refresh was reachable only from Deployment › Providers, so a workspace or organization
 * admin looking at Models had no way to pick up models a provider had just added. The
 * control is the same everywhere it appears: it posts the refresh, invalidates every
 * catalog-derived query, and reports the diff so the user can see what changed.
 *
 * Scope is the deployment, not the page: the model catalog is process-wide. What differs
 * per page is only whether the caller is allowed to ask (Platform Admin), which the caller
 * decides before mounting this.
 */

import { Button } from "@/components/ui/button";
import { refreshDynamicModels } from "@/hooks/use-providers";
import { useIsInstanceAdmin } from "@/hooks/use-settings-context";
import { refreshProviderModels } from "@/lib/api/edgequake/provider-offer";
import { RefreshCw } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

interface RefreshModelsButtonProps {
  /** Optional per-provider endpoint. Without a slug, the deployment-wide cache is cleared. */
  slug?: string;
  size?: "sm" | "default";
  variant?: "outline" | "ghost" | "secondary";
  className?: string;
  testId?: string;
  label?: string;
}

interface RefreshResult {
  added?: string[];
  removed?: string[];
  status?: string;
}

export function RefreshModelsButton({
  slug,
  size = "sm",
  variant = "outline",
  className,
  testId = "refresh-models",
  label,
}: RefreshModelsButtonProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const isInstanceAdmin = useIsInstanceAdmin();
  const [busy, setBusy] = useState(false);

  // Refresh is a platform operation. Anyone else gets no control rather than a 403.
  if (!isInstanceAdmin) return null;

  const run = async () => {
    setBusy(true);
    try {
      const result: RefreshResult | undefined = await (slug
        ? refreshProviderModels(slug).then((perProvider) => {
            // Per-provider refresh returns its own diff; the catalog still needs a re-read.
            return refreshDynamicModels(queryClient).then(() => perProvider);
          })
        : refreshDynamicModels(queryClient));
      const added = result?.added?.length ?? 0;
      const removed = result?.removed?.length ?? 0;
      toast.success(
        added + removed > 0
          ? t("settings.providers.refreshedDiff", "Models refreshed: +{{added}} new, −{{removed}} gone", {
              added,
              removed,
            })
          : t("settings.providers.refreshed", "Models refreshed"),
      );
    } catch (err) {
      toast.error(
        err instanceof Error && err.message
          ? err.message
          : t("settings.providers.refreshFailed", "Could not refresh models."),
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <Button
      type="button"
      size={size}
      variant={variant}
      className={className}
      disabled={busy}
      aria-busy={busy}
      data-testid={slug ? `${testId}-${slug}` : testId}
      onClick={() => void run()}
    >
      <RefreshCw className={busy ? "mr-2 h-4 w-4 animate-spin" : "mr-2 h-4 w-4"} aria-hidden="true" />
      {busy
        ? t("settings.providers.refreshing", "Refreshing…")
        : (label ?? t("settings.providers.refresh", "Refresh models"))}
    </Button>
  );
}
