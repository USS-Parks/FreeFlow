import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { Check, Mic } from "lucide-react";
import type { TranscriptionProgressEvent } from "@/bindings";
import { useSettings } from "@/hooks/useSettings";
import { formatKeyCombination } from "@/lib/utils/keyboard";
import { useOsType } from "@/hooks/useOsType";
import FreeFlowWordmark from "../icons/FreeFlowWordmark";

interface FirstDictationOnboardingProps {
  onComplete: () => void;
}

const FirstDictationOnboarding: React.FC<FirstDictationOnboardingProps> = ({
  onComplete,
}) => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const osType = useOsType();
  const [completed, setCompleted] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const shortcut = getSetting("bindings")?.transcribe?.current_binding ?? "";

  useEffect(() => {
    const unlisten = listen<TranscriptionProgressEvent>(
      "transcription-progress-event",
      ({ payload }) => {
        if (payload.stage === "completed") {
          setCompleted(true);
          setError(null);
        } else if (payload.stage === "failed") {
          setError(payload.error ?? t("onboarding.firstDictation.failed"));
        }
      },
    );
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [t]);

  return (
    <main className="h-screen w-screen flex items-center justify-center p-6">
      <div className="max-w-lg w-full flex flex-col items-center gap-6 text-center">
        <FreeFlowWordmark width={180} />
        <div
          className={`rounded-full p-5 ${completed ? "bg-emerald-500/20" : "bg-logo-primary/20"}`}
        >
          {completed ? (
            <Check className="w-10 h-10 text-emerald-400" />
          ) : (
            <Mic className="w-10 h-10 text-logo-primary" />
          )}
        </div>
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold text-text">
            {completed
              ? t("onboarding.firstDictation.successTitle")
              : t("onboarding.firstDictation.title")}
          </h1>
          <p className="text-text/70">
            {completed
              ? t("onboarding.firstDictation.successDescription")
              : t("onboarding.firstDictation.description", {
                  shortcut: formatKeyCombination(shortcut, osType),
                })}
          </p>
        </div>
        {!completed && (
          <textarea
            autoFocus
            aria-label={t("onboarding.firstDictation.testArea")}
            placeholder={t("onboarding.firstDictation.placeholder")}
            className="w-full min-h-32 resize-none rounded-xl border border-mid-gray/30 bg-white/5 p-4 text-text focus:border-logo-primary focus:outline-none"
          />
        )}
        {error && (
          <p role="alert" className="text-sm text-red-400">
            {error}
          </p>
        )}
        <button
          type="button"
          disabled={!completed}
          onClick={onComplete}
          className="w-full rounded-lg bg-logo-primary px-5 py-3 font-medium text-white enabled:hover:bg-logo-primary/90 disabled:cursor-not-allowed disabled:opacity-40"
        >
          {t("onboarding.firstDictation.finish")}
        </button>
      </div>
    </main>
  );
};

export default FirstDictationOnboarding;
