import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  commands,
  type AppBoundaryStyle,
  type AppCategory,
  type AppContextProfile,
  type ContextDiagnostics,
} from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import { Button } from "../ui/Button";
import { Dropdown } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";
import { Textarea } from "../ui/Textarea";
import { ToggleSwitch } from "../ui/ToggleSwitch";

const categories: AppCategory[] = [
  "email",
  "messaging",
  "document",
  "code",
  "terminal",
  "other",
];

const styleOptions: AppBoundaryStyle[] = ["standard", "compact", "literal"];

export const AppContextSettings: React.FC = React.memo(() => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const enabled = getSetting("app_context_enabled") ?? false;
  const denylist = getSetting("app_context_denylist") ?? [];
  const profiles = getSetting("app_context_profiles") ?? [];
  const [denylistText, setDenylistText] = useState(denylist.join("\n"));
  const [diagnostics, setDiagnostics] = useState<ContextDiagnostics | null>(
    null,
  );
  const [checking, setChecking] = useState(false);

  useEffect(() => setDenylistText(denylist.join("\n")), [denylist]);

  const updateProfiles = async (next: AppContextProfile[]) => {
    await updateSetting("app_context_profiles", next);
  };

  const updateProfile = async (
    category: AppCategory,
    patch: Partial<AppContextProfile>,
  ) => {
    const next = categories.map((name) => {
      const current = profiles.find((profile) => profile.category === name) ?? {
        category: name,
        boundary_style:
          name === "code" || name === "terminal" ? "literal" : "standard",
        freeflow_style:
          name === "code" || name === "terminal" ? "literal" : "natural",
        surrounding_text_enabled: name !== "code" && name !== "terminal",
        append_trailing_space: false,
      };
      return name === category ? { ...current, ...patch } : current;
    });
    await updateProfiles(next);
  };

  const saveDenylist = async () => {
    const next = denylistText
      .split(/\r?\n/)
      .map((entry) => entry.trim())
      .filter(Boolean);
    await updateSetting("app_context_denylist", next);
  };

  const inspectContext = async () => {
    setChecking(true);
    try {
      setDiagnostics(await commands.getContextDiagnostics());
    } catch (error) {
      toast.error(String(error));
    } finally {
      setChecking(false);
    }
  };

  return (
    <>
      <ToggleSwitch
        checked={enabled}
        onChange={(value) => updateSetting("app_context_enabled", value)}
        isUpdating={isUpdating("app_context_enabled")}
        label={t("settings.advanced.appContext.enabled.title")}
        description={t("settings.advanced.appContext.enabled.description")}
        grouped
      />

      <SettingContainer
        title={t("settings.advanced.appContext.denylist.title")}
        description={t("settings.advanced.appContext.denylist.description")}
        grouped
        layout="stacked"
      >
        <Textarea
          value={denylistText}
          onChange={(event) => setDenylistText(event.target.value)}
          onBlur={saveDenylist}
          placeholder={t("settings.advanced.appContext.denylist.placeholder")}
          aria-label={t("settings.advanced.appContext.denylist.title")}
          disabled={isUpdating("app_context_denylist")}
          className="w-full font-mono font-normal"
          variant="compact"
        />
      </SettingContainer>

      <SettingContainer
        title={t("settings.advanced.appContext.profiles.title")}
        description={t("settings.advanced.appContext.profiles.description")}
        grouped
        layout="stacked"
      >
        <div className="space-y-2">
          {categories.map((category) => {
            const profile = profiles.find(
              (candidate) => candidate.category === category,
            );
            const style =
              profile?.boundary_style ??
              (category === "code" || category === "terminal"
                ? "literal"
                : "standard");
            return (
              <div
                key={category}
                className="grid grid-cols-[minmax(5rem,1fr)_minmax(9rem,1fr)_auto_auto] items-center gap-3 rounded-md border border-mid-gray/20 px-3 py-2"
              >
                <span className="text-sm font-medium">
                  {t(`settings.advanced.appContext.categories.${category}`)}
                </span>
                <Dropdown
                  options={styleOptions.map((value) => ({
                    value,
                    label: t(`settings.advanced.appContext.styles.${value}`),
                  }))}
                  selectedValue={style}
                  onSelect={(value) =>
                    updateProfile(category, {
                      boundary_style: value as AppBoundaryStyle,
                    })
                  }
                  disabled={isUpdating("app_context_profiles")}
                  className="min-w-0"
                />
                <label className="flex items-center gap-2 text-xs">
                  <input
                    type="checkbox"
                    checked={profile?.surrounding_text_enabled ?? false}
                    onChange={(event) =>
                      updateProfile(category, {
                        surrounding_text_enabled: event.target.checked,
                      })
                    }
                    disabled={isUpdating("app_context_profiles")}
                  />
                  {t("settings.advanced.appContext.profiles.context")}
                </label>
                <label className="flex items-center gap-2 text-xs">
                  <input
                    type="checkbox"
                    checked={profile?.append_trailing_space ?? false}
                    onChange={(event) =>
                      updateProfile(category, {
                        append_trailing_space: event.target.checked,
                      })
                    }
                    disabled={isUpdating("app_context_profiles")}
                  />
                  {t("settings.advanced.appContext.profiles.trailingSpace")}
                </label>
              </div>
            );
          })}
        </div>
      </SettingContainer>

      <SettingContainer
        title={t("settings.advanced.appContext.diagnostics.title")}
        description={t("settings.advanced.appContext.diagnostics.description")}
        grouped
        layout="stacked"
      >
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0 text-xs text-mid-gray">
            {diagnostics ? (
              <dl className="grid grid-cols-[auto_1fr] gap-x-2 gap-y-1">
                <dt>
                  {t("settings.advanced.appContext.diagnostics.application")}
                </dt>
                <dd className="truncate text-text">
                  {diagnostics.application_id ?? t("common.unknown")}
                </dd>
                <dt>
                  {t("settings.advanced.appContext.diagnostics.category")}
                </dt>
                <dd className="text-text">
                  {t(
                    `settings.advanced.appContext.categories.${diagnostics.category}`,
                  )}
                </dd>
                <dt>{t("settings.advanced.appContext.diagnostics.status")}</dt>
                <dd className="text-text">
                  {t(
                    `settings.advanced.appContext.status.${diagnostics.status}`,
                  )}
                </dd>
                <dt>
                  {t("settings.advanced.appContext.diagnostics.characters")}
                </dt>
                <dd className="text-text">{diagnostics.captured_characters}</dd>
              </dl>
            ) : (
              t("settings.advanced.appContext.diagnostics.empty")
            )}
          </div>
          <Button
            type="button"
            variant="secondary"
            size="sm"
            onClick={inspectContext}
            disabled={checking}
          >
            {t("settings.advanced.appContext.diagnostics.inspect")}
          </Button>
        </div>
      </SettingContainer>
    </>
  );
});

AppContextSettings.displayName = "AppContextSettings";
