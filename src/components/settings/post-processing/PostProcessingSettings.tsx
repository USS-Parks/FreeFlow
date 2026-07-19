import React, { useEffect, useState } from "react";
import { Trans, useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import {
  commands,
  type LocalTransformInstallPlan,
  type LocalTransformStatus,
  type TransformAcceleration,
  type AppCategory,
  type AppContextProfile,
  type CleanupLevel,
  type FreeFlowStyle,
} from "@/bindings";

import { Alert } from "../../ui/Alert";
import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  Textarea,
} from "@/components/ui";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";

import { ShortcutInput } from "../ShortcutInput";
import { useSettings } from "../../../hooks/useSettings";

const appCategories: AppCategory[] = [
  "email",
  "messaging",
  "document",
  "code",
  "terminal",
  "other",
];

const freeFlowStyles: FreeFlowStyle[] = [
  "natural",
  "concise",
  "warm",
  "professional",
  "literal",
];

const CleanupAndStyles: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const cleanupLevel = getSetting("cleanup_level") ?? "medium";
  const profiles = getSetting("app_context_profiles") ?? [];

  const updateStyle = async (
    category: AppCategory,
    freeflowStyle: FreeFlowStyle,
  ) => {
    const next = appCategories.map((name) => {
      const current = profiles.find((profile) => profile.category === name) ?? {
        category: name,
        boundary_style:
          name === "code" || name === "terminal" ? "literal" : "standard",
        freeflow_style:
          name === "code" || name === "terminal" ? "literal" : "natural",
        surrounding_text_enabled: name !== "code" && name !== "terminal",
        append_trailing_space: false,
      };
      return name === category
        ? { ...current, freeflow_style: freeflowStyle }
        : current;
    });
    await updateSetting("app_context_profiles", next as AppContextProfile[]);
  };

  return (
    <>
      <SettingContainer
        title={t("settings.postProcessing.cleanup.level.title")}
        description={t("settings.postProcessing.cleanup.level.description")}
        layout="horizontal"
        grouped
      >
        <Dropdown
          options={(["none", "light", "medium", "high"] as CleanupLevel[]).map(
            (value) => ({
              value,
              label: t(`settings.postProcessing.cleanup.level.${value}`),
            }),
          )}
          selectedValue={cleanupLevel}
          onSelect={(value) =>
            updateSetting("cleanup_level", value as CleanupLevel)
          }
          disabled={isUpdating("cleanup_level")}
        />
      </SettingContainer>
      <SettingContainer
        title={t("settings.postProcessing.cleanup.styles.title")}
        description={t("settings.postProcessing.cleanup.styles.description")}
        layout="stacked"
        grouped
      >
        <div className="space-y-2">
          {appCategories.map((category) => {
            const style =
              profiles.find((profile) => profile.category === category)
                ?.freeflow_style ??
              (category === "code" || category === "terminal"
                ? "literal"
                : "natural");
            return (
              <div
                key={category}
                className="grid grid-cols-[minmax(7rem,1fr)_minmax(10rem,1fr)] items-center gap-3 rounded-md border border-mid-gray/20 px-3 py-2"
              >
                <span className="text-sm font-medium">
                  {t(`settings.advanced.appContext.categories.${category}`)}
                </span>
                <Dropdown
                  options={freeFlowStyles.map((value) => ({
                    value,
                    label: t(`settings.postProcessing.cleanup.styles.${value}`),
                  }))}
                  selectedValue={style}
                  onSelect={(value) =>
                    updateStyle(category, value as FreeFlowStyle)
                  }
                  disabled={isUpdating("app_context_profiles")}
                />
              </div>
            );
          })}
        </div>
      </SettingContainer>
    </>
  );
};

const PostProcessingSettingsApiComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const [plan, setPlan] = useState<LocalTransformInstallPlan | null>(null);
  const [status, setStatus] = useState<LocalTransformStatus | null>(null);
  const [accepted, setAccepted] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [progress, setProgress] = useState({
    phase: "",
    downloaded: 0,
    total: 0,
  });

  const refresh = async () => {
    const [planResult, statusResult] = await Promise.all([
      commands.getLocalTransformInstallPlan(),
      commands.getLocalTransformStatus(),
    ]);
    if (planResult.status === "ok") setPlan(planResult.data);
    if (statusResult.status === "ok") setStatus(statusResult.data);
  };

  useEffect(() => {
    void refresh();
    const unlisten = listen<{
      phase: string;
      downloaded_bytes: number;
      total_bytes: number;
    }>("local-transform-install-progress", ({ payload }) => {
      setProgress({
        phase: payload.phase,
        downloaded: payload.downloaded_bytes,
        total: payload.total_bytes,
      });
    });
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, []);

  const install = async () => {
    if (!accepted || !plan) return;
    setBusy(true);
    setError("");
    const result = await commands.installLocalTransform(plan.manifest_digest);
    if (result.status === "ok") {
      setStatus(result.data);
      setAccepted(false);
    } else {
      setError(result.error);
    }
    setBusy(false);
    await refresh();
  };

  const cancel = async () => {
    await commands.cancelLocalTransformInstall();
    setBusy(false);
    await refresh();
  };

  const remove = async () => {
    const result = await commands.deleteLocalTransformInstall();
    if (result.status === "error") setError(result.error);
    await refresh();
  };

  const formatBytes = (bytes: number) =>
    `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  const acceleration =
    (getSetting("local_transform_acceleration") as TransformAcceleration) ||
    "auto";
  const timeout = getSetting("local_transform_timeout_seconds") || 30;

  return (
    <>
      <SettingContainer
        title={t("settings.postProcessing.localRuntime.title")}
        description={t("settings.postProcessing.localRuntime.description")}
        layout="stacked"
        grouped={true}
      >
        <div className="space-y-3 text-sm">
          {plan && (
            <div className="space-y-1 rounded-md border border-mid-gray/20 p-3">
              <p className="font-semibold">
                {t("settings.postProcessing.localRuntime.package", {
                  size: formatBytes(plan.total_download_bytes),
                })}
              </p>
              {[plan.runtime, plan.model].map((artifact) => (
                <div
                  key={artifact.filename}
                  className="space-y-1 border-t border-mid-gray/20 pt-2"
                >
                  <p className="font-semibold">{artifact.filename}</p>
                  <p>{artifact.size_bytes.toLocaleString()}</p>
                  <p className="break-all text-xs text-mid-gray">
                    {artifact.source_url}
                  </p>
                  <p className="break-all text-xs text-mid-gray">
                    {artifact.destination}
                  </p>
                  <p className="break-all text-xs text-mid-gray">
                    {artifact.sha256}
                  </p>
                  {artifact.licenses.map((license) => (
                    <div
                      key={`${artifact.filename}-${license.scope}`}
                      className="text-xs text-mid-gray"
                    >
                      <p>
                        {license.name} ({license.identifier})
                      </p>
                      <p>{license.attribution}</p>
                      <p className="break-all">{license.url}</p>
                    </div>
                  ))}
                </div>
              ))}
              <p className="text-xs">{plan.recommendation.message}</p>
            </div>
          )}
          <p>
            {status?.installed
              ? t("settings.postProcessing.localRuntime.installed")
              : t("settings.postProcessing.localRuntime.notInstalled")}
          </p>
          {(busy || status?.installing) && progress.total > 0 && (
            <p>
              {t("settings.postProcessing.localRuntime.progress", {
                phase: progress.phase,
                downloaded: formatBytes(progress.downloaded),
                total: formatBytes(progress.total),
              })}
            </p>
          )}
          {!status?.installed && (
            <label className="flex items-start gap-2">
              <input
                type="checkbox"
                checked={accepted}
                onChange={(event) => setAccepted(event.target.checked)}
              />
              <span>{t("settings.postProcessing.localRuntime.consent")}</span>
            </label>
          )}
          <div className="flex gap-2">
            {!status?.installed && !busy && !status?.installing && (
              <Button
                onClick={install}
                variant="primary"
                size="md"
                disabled={!accepted || !plan}
              >
                {t("settings.postProcessing.localRuntime.install")}
              </Button>
            )}
            {(busy || status?.installing) && (
              <Button onClick={cancel} variant="secondary" size="md">
                {t("settings.postProcessing.localRuntime.cancel")}
              </Button>
            )}
            {status?.installed && (
              <Button onClick={remove} variant="secondary" size="md">
                {t("settings.postProcessing.localRuntime.remove")}
              </Button>
            )}
          </div>
          {error && (
            <Alert variant="error" contained>
              {error}
            </Alert>
          )}
        </div>
      </SettingContainer>
      <SettingContainer
        title={t("settings.postProcessing.localRuntime.acceleration")}
        description={t("settings.postProcessing.localRuntime.description")}
        layout="horizontal"
        grouped={true}
      >
        <Dropdown
          options={[
            {
              value: "auto",
              label: t("settings.postProcessing.localRuntime.auto"),
            },
            { value: "cpu", label: "CPU" },
            {
              value: "gpu",
              label: t("settings.postProcessing.localRuntime.gpu"),
            },
          ]}
          selectedValue={acceleration}
          onSelect={(value) =>
            updateSetting(
              "local_transform_acceleration",
              value as TransformAcceleration,
            )
          }
          disabled={isUpdating("local_transform_acceleration")}
        />
      </SettingContainer>
      <SettingContainer
        title={t("settings.postProcessing.localRuntime.timeout")}
        description={t("settings.postProcessing.localRuntime.description")}
        layout="horizontal"
        grouped={true}
      >
        <Input
          type="number"
          min={5}
          max={120}
          value={timeout}
          onChange={(event) => {
            const value = Number(event.target.value);
            if (value >= 5 && value <= 120)
              void updateSetting("local_transform_timeout_seconds", value);
          }}
          disabled={isUpdating("local_transform_timeout_seconds")}
          className="w-24"
        />
      </SettingContainer>
    </>
  );
};

const PostProcessingSettingsPromptsComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating, refreshSettings } =
    useSettings();
  const [isCreating, setIsCreating] = useState(false);
  const [draftName, setDraftName] = useState("");
  const [draftText, setDraftText] = useState("");

  const prompts = getSetting("post_process_prompts") || [];
  const selectedPromptId = getSetting("post_process_selected_prompt_id") || "";
  const selectedPrompt =
    prompts.find((prompt) => prompt.id === selectedPromptId) || null;

  useEffect(() => {
    if (isCreating) return;

    if (selectedPrompt) {
      setDraftName(selectedPrompt.name);
      setDraftText(selectedPrompt.prompt);
    } else {
      setDraftName("");
      setDraftText("");
    }
  }, [
    isCreating,
    selectedPromptId,
    selectedPrompt?.name,
    selectedPrompt?.prompt,
  ]);

  const handlePromptSelect = (promptId: string | null) => {
    if (!promptId) return;
    updateSetting("post_process_selected_prompt_id", promptId);
    setIsCreating(false);
  };

  const handleCreatePrompt = async () => {
    if (!draftName.trim() || !draftText.trim()) return;

    try {
      const result = await commands.addPostProcessPrompt(
        draftName.trim(),
        draftText.trim(),
      );
      if (result.status === "ok") {
        await refreshSettings();
        updateSetting("post_process_selected_prompt_id", result.data.id);
        setIsCreating(false);
      }
    } catch (error) {
      console.error("Failed to create prompt:", error);
    }
  };

  const handleUpdatePrompt = async () => {
    if (!selectedPromptId || !draftName.trim() || !draftText.trim()) return;

    try {
      await commands.updatePostProcessPrompt(
        selectedPromptId,
        draftName.trim(),
        draftText.trim(),
      );
      await refreshSettings();
    } catch (error) {
      console.error("Failed to update prompt:", error);
    }
  };

  const handleDeletePrompt = async (promptId: string) => {
    if (!promptId) return;

    try {
      await commands.deletePostProcessPrompt(promptId);
      await refreshSettings();
      setIsCreating(false);
    } catch (error) {
      console.error("Failed to delete prompt:", error);
    }
  };

  const handleCancelCreate = () => {
    setIsCreating(false);
    if (selectedPrompt) {
      setDraftName(selectedPrompt.name);
      setDraftText(selectedPrompt.prompt);
    } else {
      setDraftName("");
      setDraftText("");
    }
  };

  const handleStartCreate = () => {
    setIsCreating(true);
    setDraftName("");
    setDraftText("");
  };

  const hasPrompts = prompts.length > 0;
  const isDirty =
    !!selectedPrompt &&
    (draftName.trim() !== selectedPrompt.name ||
      draftText.trim() !== selectedPrompt.prompt.trim());

  return (
    <SettingContainer
      title={t("settings.postProcessing.prompts.selectedPrompt.title")}
      description={t(
        "settings.postProcessing.prompts.selectedPrompt.description",
      )}
      descriptionMode="tooltip"
      layout="stacked"
      grouped={true}
    >
      <div className="space-y-3">
        <div className="flex gap-2 min-w-0">
          <Dropdown
            selectedValue={selectedPromptId || null}
            options={prompts.map((p) => ({
              value: p.id,
              label: p.name,
            }))}
            onSelect={(value) => handlePromptSelect(value)}
            placeholder={
              prompts.length === 0
                ? t("settings.postProcessing.prompts.noPrompts")
                : t("settings.postProcessing.prompts.selectPrompt")
            }
            disabled={
              isUpdating("post_process_selected_prompt_id") || isCreating
            }
            className="flex-1 min-w-0"
          />
          <Button
            onClick={handleStartCreate}
            variant="primary"
            size="md"
            disabled={isCreating}
            className="shrink-0"
          >
            {t("settings.postProcessing.prompts.createNew")}
          </Button>
        </div>

        {!isCreating && hasPrompts && selectedPrompt && (
          <div className="space-y-3">
            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.postProcessing.prompts.promptLabel")}
              </label>
              <Input
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                placeholder={t(
                  "settings.postProcessing.prompts.promptLabelPlaceholder",
                )}
                variant="compact"
              />
            </div>

            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.postProcessing.prompts.promptInstructions")}
              </label>
              <Textarea
                value={draftText}
                onChange={(e) => setDraftText(e.target.value)}
                placeholder={t(
                  "settings.postProcessing.prompts.promptInstructionsPlaceholder",
                )}
              />
              <p className="text-xs text-mid-gray/70">
                <Trans
                  i18nKey="settings.postProcessing.prompts.promptTip"
                  components={{ code: <code /> }}
                />
              </p>
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                onClick={handleUpdatePrompt}
                variant="primary"
                size="md"
                disabled={!draftName.trim() || !draftText.trim() || !isDirty}
              >
                {t("settings.postProcessing.prompts.updatePrompt")}
              </Button>
              <Button
                onClick={() => handleDeletePrompt(selectedPromptId)}
                variant="secondary"
                size="md"
                disabled={!selectedPromptId || prompts.length <= 1}
              >
                {t("settings.postProcessing.prompts.deletePrompt")}
              </Button>
            </div>
          </div>
        )}

        {!isCreating && !selectedPrompt && (
          <div className="p-3 bg-mid-gray/5 rounded-md border border-mid-gray/20">
            <p className="text-sm text-mid-gray">
              {hasPrompts
                ? t("settings.postProcessing.prompts.selectToEdit")
                : t("settings.postProcessing.prompts.createFirst")}
            </p>
          </div>
        )}

        {isCreating && (
          <div className="space-y-3">
            <div className="space-y-2 block flex flex-col">
              <label className="text-sm font-semibold text-text">
                {t("settings.postProcessing.prompts.promptLabel")}
              </label>
              <Input
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                placeholder={t(
                  "settings.postProcessing.prompts.promptLabelPlaceholder",
                )}
                variant="compact"
              />
            </div>

            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.postProcessing.prompts.promptInstructions")}
              </label>
              <Textarea
                value={draftText}
                onChange={(e) => setDraftText(e.target.value)}
                placeholder={t(
                  "settings.postProcessing.prompts.promptInstructionsPlaceholder",
                )}
              />
              <p className="text-xs text-mid-gray/70">
                <Trans
                  i18nKey="settings.postProcessing.prompts.promptTip"
                  components={{ code: <code /> }}
                />
              </p>
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                onClick={handleCreatePrompt}
                variant="primary"
                size="md"
                disabled={!draftName.trim() || !draftText.trim()}
              >
                {t("settings.postProcessing.prompts.createPrompt")}
              </Button>
              <Button
                onClick={handleCancelCreate}
                variant="secondary"
                size="md"
              >
                {t("settings.postProcessing.prompts.cancel")}
              </Button>
            </div>
          </div>
        )}
      </div>
    </SettingContainer>
  );
};

export const PostProcessingSettingsApi = React.memo(
  PostProcessingSettingsApiComponent,
);
PostProcessingSettingsApi.displayName = "PostProcessingSettingsApi";

export const PostProcessingSettingsPrompts = React.memo(
  PostProcessingSettingsPromptsComponent,
);
PostProcessingSettingsPrompts.displayName = "PostProcessingSettingsPrompts";

export const PostProcessingSettings: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.postProcessing.hotkey.title")}>
        <ShortcutInput
          shortcutId="transcribe_with_post_process"
          descriptionMode="tooltip"
          grouped={true}
        />
      </SettingsGroup>

      <SettingsGroup title={t("settings.postProcessing.cleanup.title")}>
        <CleanupAndStyles />
      </SettingsGroup>

      <SettingsGroup
        title={t("settings.postProcessing.localRuntime.groupTitle")}
      >
        <PostProcessingSettingsApi />
      </SettingsGroup>
    </div>
  );
};
