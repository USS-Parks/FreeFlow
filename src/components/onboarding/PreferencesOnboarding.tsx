import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { platform } from "@tauri-apps/plugin-os";
import {
  checkAccessibilityPermission,
  checkMicrophonePermission,
  requestAccessibilityPermission,
  requestMicrophonePermission,
} from "tauri-plugin-macos-permissions-api";
import { FolderOpen, RefreshCw } from "lucide-react";
import { commands, type OnboardingDiagnostics } from "@/bindings";
import { GlobalShortcutInput } from "../settings/GlobalShortcutInput";
import { AutostartToggle } from "../settings/AutostartToggle";
import FreeFlowWordmark from "../icons/FreeFlowWordmark";

interface PreferencesOnboardingProps {
  onContinue: () => void;
}

const PreferencesOnboarding: React.FC<PreferencesOnboardingProps> = ({
  onContinue,
}) => {
  const { t } = useTranslation();
  const [diagnostics, setDiagnostics] = useState<OnboardingDiagnostics | null>(
    null,
  );
  const [diagnosticError, setDiagnosticError] = useState<string | null>(null);
  const [permissionDiagnostics, setPermissionDiagnostics] = useState({
    microphone: false,
    accessibility: platform() !== "macos",
  });

  const refreshDiagnostics = async () => {
    try {
      const currentPlatform = platform();
      const [result, microphone, accessibility] = await Promise.all([
        commands.getOnboardingDiagnostics(),
        currentPlatform === "macos"
          ? checkMicrophonePermission()
          : currentPlatform === "windows"
            ? commands
                .getWindowsMicrophonePermissionStatus()
                .then(
                  (status) =>
                    !status.supported || status.overall_access !== "denied",
                )
            : Promise.resolve(true),
        currentPlatform === "macos"
          ? checkAccessibilityPermission()
          : Promise.resolve(true),
      ]);
      if (result.status === "ok") {
        setDiagnostics(result.data);
        setDiagnosticError(null);
      } else {
        setDiagnosticError(result.error);
      }
      setPermissionDiagnostics({ microphone, accessibility });
    } catch (error) {
      setDiagnosticError(String(error));
    }
  };

  const repairPermissions = async () => {
    if (platform() === "windows") {
      await commands.openMicrophonePrivacySettings();
    } else if (platform() === "macos") {
      if (!permissionDiagnostics.microphone) {
        await requestMicrophonePermission();
      }
      if (!permissionDiagnostics.accessibility) {
        await requestAccessibilityPermission();
      }
    }
    await refreshDiagnostics();
  };

  useEffect(() => {
    void refreshDiagnostics();
  }, []);

  return (
    <main className="h-screen w-screen overflow-y-auto p-6">
      <div className="max-w-2xl mx-auto flex flex-col items-center gap-6">
        <FreeFlowWordmark width={180} />
        <div className="text-center space-y-2">
          <h1 className="text-2xl font-semibold text-text">
            {t("onboarding.preferences.title")}
          </h1>
          <p className="text-text/70">
            {t("onboarding.preferences.description")}
          </p>
        </div>
        <section className="w-full rounded-xl border border-mid-gray/20 overflow-hidden">
          <GlobalShortcutInput
            shortcutId="transcribe"
            descriptionMode="inline"
            grouped
          />
          <AutostartToggle descriptionMode="inline" grouped />
        </section>
        <section className="w-full rounded-xl border border-mid-gray/20 bg-white/5 p-4 space-y-3">
          <div className="flex items-center justify-between gap-3">
            <div>
              <h2 className="font-medium text-text">
                {t("onboarding.preferences.diagnosticsTitle")}
              </h2>
              <p className="text-sm text-text/60">
                {t("onboarding.preferences.diagnosticsDescription")}
              </p>
            </div>
            <button
              type="button"
              onClick={refreshDiagnostics}
              aria-label={t("onboarding.preferences.refreshDiagnostics")}
              className="rounded-md p-2 hover:bg-white/10"
            >
              <RefreshCw className="w-4 h-4" />
            </button>
          </div>
          {diagnostics && (
            <dl className="grid gap-1 text-sm">
              <div className="flex justify-between gap-4">
                <dt className="text-text/60">
                  {t("onboarding.preferences.model")}
                </dt>
                <dd>
                  {diagnostics.model_selected
                    ? t("common.ready")
                    : t("common.needsAttention")}
                </dd>
              </div>
              <div className="flex justify-between gap-4">
                <dt className="text-text/60">
                  {t("onboarding.preferences.autostart")}
                </dt>
                <dd>
                  {diagnostics.autostart_enabled
                    ? t("common.enabled")
                    : t("common.disabled")}
                </dd>
              </div>
              <div className="flex justify-between gap-4">
                <dt className="text-text/60">
                  {t("onboarding.preferences.microphone")}
                </dt>
                <dd>
                  {permissionDiagnostics.microphone
                    ? t("common.ready")
                    : t("common.needsAttention")}
                </dd>
              </div>
              {platform() === "macos" && (
                <div className="flex justify-between gap-4">
                  <dt className="text-text/60">
                    {t("onboarding.preferences.accessibility")}
                  </dt>
                  <dd>
                    {permissionDiagnostics.accessibility
                      ? t("common.ready")
                      : t("common.needsAttention")}
                  </dd>
                </div>
              )}
            </dl>
          )}
          {diagnosticError && (
            <p role="alert" className="text-sm text-red-400">
              {diagnosticError}
            </p>
          )}
          {(!permissionDiagnostics.microphone ||
            !permissionDiagnostics.accessibility) && (
            <button
              type="button"
              onClick={repairPermissions}
              className="text-sm font-medium text-logo-primary hover:underline"
            >
              {t("onboarding.preferences.repairPermissions")}
            </button>
          )}
          <button
            type="button"
            onClick={() => commands.openAppDataDir()}
            className="flex items-center gap-2 text-sm text-logo-primary hover:underline"
          >
            <FolderOpen className="w-4 h-4" />
            {t("onboarding.preferences.openData")}
          </button>
          <p className="text-xs text-text/50">
            {t("onboarding.preferences.uninstallData")}
          </p>
        </section>
        <button
          type="button"
          onClick={onContinue}
          className="w-full rounded-lg bg-logo-primary px-5 py-3 font-medium text-white hover:bg-logo-primary/90"
        >
          {t("onboarding.preferences.continue")}
        </button>
      </div>
    </main>
  );
};

export default PreferencesOnboarding;
