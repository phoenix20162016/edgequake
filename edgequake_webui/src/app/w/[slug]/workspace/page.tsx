/**
 * @module WorkspacePage (Deeplink)
 * @description Workspace configuration page accessible via /w/[slug]/workspace deeplink.
 *
 * @implements SPEC-032: Workspace configuration via deeplink
 * @implements SPEC-101 LAW-101-12: Same reconfigure wizard as /workspace (parity)
 * @implements FEAT0802: Workspace detail view with LLM/embedding configuration (deeplink route)
 * @implements UC0305: User views workspace configuration
 *
 * @enforces BR0305: Workspace config is visible and editable
 * @enforces BR0306: Rebuild action available when model changes
 */
'use client';

import { useParams } from 'next/navigation';

import { ReconfigureWorkspaceWizard } from '@/components/onboarding/reconfigure-workspace-wizard';
import {
  WorkspaceLoading,
  WorkspaceNotFound,
} from '@/components/workspace/workspace-deeplink-states';
import { WorkspaceEntityTypesCard } from '@/components/workspace/workspace-entity-types-card';
import { WorkspaceRelationTypesCard } from '@/components/workspace/workspace-relation-types-card';
import { WorkspaceExtractionLanguageCard } from '@/components/workspace/workspace-extraction-language-card';
import { WorkspaceChunkingCard } from '@/components/workspace/workspace-chunking-card';
import { WorkspaceExtractBudgetCard } from '@/components/workspace/workspace-extract-budget-card';
import { WorkspaceExtractionModeCard } from '@/components/workspace/workspace-extraction-mode-card';
import { WorkspacePageHeader } from '@/components/workspace/workspace-page-header';
import { WorkspaceActionsCard } from '@/components/workspace/workspace-actions-card';
import { WorkspaceExtendedModelConfig } from '@/components/workspace/workspace-extended-model-config';
import { WorkspaceModelConfigGrid } from '@/components/workspace/workspace-model-config-grid';
import { WorkspaceStatusFooter } from '@/components/workspace/workspace-status-footer';
import { WorkspaceStatsCards } from '@/components/workspace/workspace-stats-cards';
import { ENTITY_PRESETS } from '@/constants/entity-presets';
import { useWorkspaceDetailQueries } from '@/hooks/use-workspace-detail-queries';
import { useWorkspaceSlugResolver } from '@/hooks/use-workspace-slug-resolver';
import { Card, CardContent } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Skeleton } from '@/components/ui/skeleton';
import type { WorkspaceRebuildHints } from '@/lib/onboarding/workspace-config-diff';
import { getWorkspacePdfParserBackend } from '@/lib/workspace/drafts';
import { useTenantStore } from '@/stores/use-tenant-store';
import type { Workspace } from '@/types';
import { useQueryClient } from '@tanstack/react-query';
import {
  AlertTriangle,
  FolderKanban,
} from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

export default function WorkspacePage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const params = useParams();
  const slug = params?.slug as string;
  const { isLoading: resolvingSlug, error: slugError, isReady } =
    useWorkspaceSlugResolver(slug);
  const { selectedTenantId, selectedWorkspaceId } = useTenantStore();

  const [reconfigureOpen, setReconfigureOpen] = useState(false);
  const [pendingRebuild, setPendingRebuild] = useState<WorkspaceRebuildHints | null>(null);

  const {
    workspace,
    stats,
    isLoadingWorkspace,
    isLoadingStats,
    refetchWorkspace,
  } = useWorkspaceDetailQueries(selectedTenantId, selectedWorkspaceId, {
    enabled: isReady,
  });

  const documentCount = stats?.document_count ?? workspace?.document_count ?? 0;

  const handleReconfigureApplied = (result: {
    workspace: Workspace;
    pendingRebuild: WorkspaceRebuildHints | null;
    extractionLanguageChanged: boolean;
  }) => {
    toast.success(t('workspace.updateSuccess', 'Workspace updated successfully'));
    queryClient.setQueryData(
      ['workspace', selectedTenantId, selectedWorkspaceId],
      result.workspace,
    );
    queryClient.invalidateQueries({
      queryKey: ['workspace', selectedTenantId, selectedWorkspaceId],
    });
    if (result.extractionLanguageChanged) {
      toast.info(
        t(
          'workspace.extractionLanguage.changedToast',
          'Extraction language updated. Reprocess documents to refresh the graph.',
        ),
        { duration: 6000 },
      );
    }
    if (result.pendingRebuild) {
      setPendingRebuild(result.pendingRebuild);
      const { embeddings, extraction, vision } = result.pendingRebuild;
      if (embeddings && extraction) {
        toast.info(t('workspace.rebuildRequired', 'Model changes detected'), {
          description: t(
            'workspace.rebuildBothHint',
            'Both embedding and LLM models changed. Use "Rebuild Embeddings" to reprocess all documents.',
          ),
          duration: 8000,
        });
      } else if (embeddings) {
        toast.info(t('workspace.embeddingRebuildRequired', 'Embedding model changed'), {
          description: t(
            'workspace.embeddingRebuildHint',
            'Use "Rebuild Embeddings" to regenerate vector embeddings with the new model.',
          ),
          duration: 6000,
        });
      } else if (extraction) {
        toast.info(t('workspace.llmRebuildRequired', 'LLM model changed'), {
          description: t(
            'workspace.llmRebuildHint',
            'Use "Rebuild Knowledge Graph" to re-extract entities with the new LLM model.',
          ),
          duration: 6000,
        });
      } else if (vision) {
        toast.info(t('workspace.visionRebuildRequired', 'Vision LLM model changed'), {
          description: t(
            'workspace.visionRebuildHint',
            'Use "Rebuild Knowledge Graph" to re-extract PDF documents with the new vision model from original files.',
          ),
          duration: 6000,
        });
      }
    }
  };

  if (resolvingSlug || !isReady) {
    return <WorkspaceLoading context="workspace configuration" />;
  }

  if (slugError) {
    return (
      <WorkspaceNotFound
        slug={slug}
        fallbackHref="/workspace"
        fallbackLabel={t('workspace.goToSettings', 'Go to Workspace Settings')}
      />
    );
  }

  if (!selectedTenantId || !selectedWorkspaceId) {
    return (
      <div className="container mx-auto p-page">
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <FolderKanban className="h-12 w-12 text-muted-foreground mb-4" />
            <h2 className="text-lg font-medium text-muted-foreground">
              {t('workspace.noWorkspaceSelected', 'No Workspace Selected')}
            </h2>
            <p className="text-sm text-muted-foreground mt-2">
              {t(
                'workspace.selectWorkspaceHint',
                'Please select a workspace from the sidebar.',
              )}
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (isLoadingWorkspace) {
    return (
      <div className="container mx-auto p-page space-y-page">
        <Skeleton className="h-8 w-64" />
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {[...Array(4)].map((_, i) => (
            <Skeleton key={i} className="h-32" />
          ))}
        </div>
        <Skeleton className="h-64" />
      </div>
    );
  }

  if (!workspace) {
    return (
      <div className="container mx-auto p-page">
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <AlertTriangle className="h-12 w-12 text-destructive mb-4" />
            <h2 className="text-lg font-medium">
              {t('workspace.notFound', 'Workspace Not Found')}
            </h2>
            <p className="text-sm text-muted-foreground mt-2">
              {t(
                'workspace.notFoundHint',
                'The selected workspace could not be loaded.',
              )}
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  const entityTypes = workspace.entity_types?.length
    ? workspace.entity_types
    : [...ENTITY_PRESETS.general.types];

  return (
    <div className="container mx-auto space-y-page p-page">
      <WorkspacePageHeader
        workspace={workspace}
        onRefresh={() => refetchWorkspace()}
        onEditStart={() => setReconfigureOpen(true)}
      />

      <Separator />

      <WorkspaceStatsCards
        workspace={workspace}
        stats={stats}
        isLoadingStats={isLoadingStats}
      />

      <WorkspaceModelConfigGrid
        workspace={workspace}
        isEditing={false}
        selectedLLM={undefined}
        selectedEmbedding={undefined}
        onLlmChange={() => {}}
        onEmbeddingChange={() => {}}
        llmModelChanged={false}
        embeddingModelChanged={false}
      />

      <WorkspaceExtendedModelConfig
        workspace={workspace}
        isEditing={false}
        selectedVisionLLM={undefined}
        selectedPdfParserBackend={getWorkspacePdfParserBackend(workspace)}
        onVisionLlmChange={() => {}}
        onPdfParserBackendChange={() => {}}
        visionLLMChanged={false}
      />

      <WorkspaceExtractionLanguageCard
        isEditing={false}
        workspace={workspace}
        selectedLanguage={workspace.extraction_language ?? null}
        onLanguageChange={() => {}}
      />

      <WorkspaceChunkingCard isEditing={false} workspace={workspace} />
      <WorkspaceExtractBudgetCard isEditing={false} workspace={workspace} />
      <WorkspaceExtractionModeCard workspace={workspace} />

      <div
        className="grid grid-cols-1 gap-4 lg:grid-cols-2 lg:items-start"
        data-testid="workspace-kg-schema-row"
      >
        <WorkspaceEntityTypesCard
          isEditing={false}
          workspace={workspace}
          selectedTypes={entityTypes}
          onTypesChange={() => {}}
          strictLimit={workspace.entity_types_strict ?? true}
          onStrictLimitChange={() => {}}
          extractionLanguage={workspace.extraction_language ?? null}
        />

        <WorkspaceRelationTypesCard workspace={workspace} />
      </div>

      <WorkspaceActionsCard
        workspace={workspace}
        pendingRebuild={pendingRebuild}
        includeVisionPending
        onRebuildComplete={() => {
          queryClient.invalidateQueries({
            queryKey: ['workspaceStats', selectedWorkspaceId],
          });
          queryClient.invalidateQueries({ queryKey: ['documents'] });
          setPendingRebuild(null);
        }}
      />

      <WorkspaceStatusFooter />

      <ReconfigureWorkspaceWizard
        open={reconfigureOpen}
        onOpenChange={setReconfigureOpen}
        tenantId={selectedTenantId}
        workspace={workspace}
        documentCount={documentCount}
        onApplied={handleReconfigureApplied}
      />
    </div>
  );
}
