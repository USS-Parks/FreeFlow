import React, { useCallback, useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  Ban,
  Check,
  Copy,
  FolderOpen,
  Search,
  RotateCcw,
  Star,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  commands,
  events,
  type HistoryEntry,
  type HistoryUpdatePayload,
} from "@/bindings";
import { useOsType } from "@/hooks/useOsType";
import { useSettings } from "@/hooks/useSettings";
import { formatDateTime } from "@/utils/dateFormat";
import { AudioPlayer, AudioPlayerGroup } from "../../ui/AudioPlayer";
import { Button } from "../../ui/Button";

const IconButton: React.FC<{
  onClick: () => void;
  title: string;
  disabled?: boolean;
  active?: boolean;
  children: React.ReactNode;
}> = ({ onClick, title, disabled, active, children }) => (
  <button
    onClick={onClick}
    disabled={disabled}
    className={`p-1.5 rounded-md flex items-center justify-center transition-colors cursor-pointer disabled:cursor-not-allowed disabled:text-text/20 ${
      active
        ? "text-logo-primary hover:text-logo-primary/80"
        : "text-text/50 hover:text-logo-primary"
    }`}
    title={title}
  >
    {children}
  </button>
);

const PAGE_SIZE = 30;

interface OpenRecordingsButtonProps {
  onClick: () => void;
  label: string;
}

const OpenRecordingsButton: React.FC<OpenRecordingsButtonProps> = ({
  onClick,
  label,
}) => (
  <Button
    onClick={onClick}
    variant="secondary"
    size="sm"
    className="flex items-center gap-2"
    title={label}
  >
    <FolderOpen className="w-4 h-4" />
    <span>{label}</span>
  </Button>
);

export const HistorySettings: React.FC = () => {
  const { t } = useTranslation();
  const osType = useOsType();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [hasMore, setHasMore] = useState(true);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const entriesRef = useRef<HistoryEntry[]>([]);
  const loadingRef = useRef(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [deferredQuery, setDeferredQuery] = useState("");
  const storageMode = getSetting("history_storage_mode") ?? "store";

  useEffect(() => {
    const timeout = window.setTimeout(
      () => setDeferredQuery(searchQuery.trim()),
      250,
    );
    return () => window.clearTimeout(timeout);
  }, [searchQuery]);

  // Keep ref in sync for use in IntersectionObserver callback
  useEffect(() => {
    entriesRef.current = entries;
  }, [entries]);

  const loadPage = useCallback(
    async (cursor?: number) => {
      const isFirstPage = cursor === undefined;
      if (!isFirstPage && loadingRef.current) return;
      loadingRef.current = true;

      if (isFirstPage) setLoading(true);

      try {
        const result = await commands.getHistoryEntries(
          cursor ?? null,
          PAGE_SIZE,
          deferredQuery || null,
        );
        if (result.status === "ok") {
          const { entries: newEntries, has_more } = result.data;
          setEntries((prev) =>
            isFirstPage ? newEntries : [...prev, ...newEntries],
          );
          setHasMore(has_more);
        }
      } catch (error) {
        console.error("Failed to load history entries:", error);
      } finally {
        setLoading(false);
        loadingRef.current = false;
      }
    },
    [deferredQuery],
  );

  // Initial load
  useEffect(() => {
    loadPage();
  }, [loadPage]);

  // Infinite scroll via IntersectionObserver
  useEffect(() => {
    if (loading) return;

    const sentinel = sentinelRef.current;
    if (!sentinel || !hasMore) return;

    const observer = new IntersectionObserver(
      (observerEntries) => {
        const first = observerEntries[0];
        if (first.isIntersecting) {
          const lastEntry = entriesRef.current[entriesRef.current.length - 1];
          if (lastEntry) {
            loadPage(lastEntry.id);
          }
        }
      },
      { threshold: 0 },
    );

    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [loading, hasMore, loadPage]);

  // Listen for new entries added from the transcription pipeline
  useEffect(() => {
    const unlisten = events.historyUpdatePayload.listen((event) => {
      const payload: HistoryUpdatePayload = event.payload;
      if (payload.action === "added") {
        if (!deferredQuery) {
          setEntries((prev) => [payload.entry, ...prev]);
        }
      } else if (payload.action === "updated") {
        setEntries((prev) =>
          prev.map((e) => (e.id === payload.entry.id ? payload.entry : e)),
        );
      } else if (payload.action === "deleted") {
        setEntries((prev) => prev.filter((entry) => entry.id !== payload.id));
      } else if (payload.action === "cleared") setEntries([]);
      // "toggled" is handled by the optimistic save update.
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [deferredQuery]);

  const toggleSaved = async (id: number) => {
    // Optimistic update
    setEntries((prev) =>
      prev.map((e) => (e.id === id ? { ...e, saved: !e.saved } : e)),
    );
    try {
      const result = await commands.toggleHistoryEntrySaved(id);
      if (result.status !== "ok") {
        // Revert on failure
        setEntries((prev) =>
          prev.map((e) => (e.id === id ? { ...e, saved: !e.saved } : e)),
        );
      }
    } catch (error) {
      console.error("Failed to toggle saved status:", error);
      // Revert on failure
      setEntries((prev) =>
        prev.map((e) => (e.id === id ? { ...e, saved: !e.saved } : e)),
      );
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      const result = await commands.copyTextToClipboard(text);
      if (result.status !== "ok") throw new Error(String(result.error));
    } catch (error) {
      console.error("Failed to copy to clipboard:", error);
    }
  };

  const getAudioUrl = useCallback(
    async (fileName: string) => {
      try {
        const result = await commands.getAudioFilePath(fileName);
        if (result.status === "ok") {
          if (osType === "linux") {
            const fileResult = await commands.readAudioFile(fileName);
            if (fileResult.status !== "ok") {
              throw new Error(String(fileResult.error));
            }
            const fileData = new Uint8Array(fileResult.data);
            const blob = new Blob([fileData], { type: "audio/wav" });
            return URL.createObjectURL(blob);
          }
          return convertFileSrc(result.data, "asset");
        }
        return null;
      } catch (error) {
        console.error("Failed to get audio file path:", error);
        return null;
      }
    },
    [osType],
  );

  const deleteAudioEntry = async (id: number) => {
    // Optimistically remove
    setEntries((prev) => prev.filter((e) => e.id !== id));
    try {
      const result = await commands.deleteHistoryEntry(id);
      if (result.status !== "ok") {
        // Reload on failure
        loadPage();
      }
    } catch (error) {
      console.error("Failed to delete entry:", error);
      loadPage();
    }
  };

  const clearHistory = async () => {
    if (!window.confirm(t("settings.history.clearConfirm"))) return;
    const result = await commands.clearHistory();
    if (result.status !== "ok") {
      toast.error(t("settings.history.clearError"));
      return;
    }
    setEntries([]);
    setHasMore(false);
    toast.success(
      t("settings.history.cleared", { count: Number(result.data) }),
    );
  };

  const toggleNeverStore = async () => {
    await updateSetting(
      "history_storage_mode",
      storageMode === "never_store" ? "store" : "never_store",
    );
  };

  const retryHistoryEntry = async (id: number) => {
    const result = await commands.retryHistoryEntryTranscription(id);
    if (result.status !== "ok") {
      throw new Error(String(result.error));
    }
  };

  const openRecordingsFolder = async () => {
    try {
      const result = await commands.openRecordingsFolder();
      if (result.status !== "ok") {
        throw new Error(String(result.error));
      }
    } catch (error) {
      console.error("Failed to open recordings folder:", error);
    }
  };

  let content: React.ReactNode;

  if (loading) {
    content = (
      <div className="px-4 py-3 text-center text-text/60">
        {t("settings.history.loading")}
      </div>
    );
  } else if (entries.length === 0) {
    content = (
      <div className="px-4 py-3 text-center text-text/60">
        {t("settings.history.empty")}
      </div>
    );
  } else {
    content = (
      <>
        <AudioPlayerGroup>
          <div className="divide-y divide-mid-gray/20">
            {entries.map((entry) => (
              <HistoryEntryComponent
                key={entry.id}
                entry={entry}
                onToggleSaved={() => toggleSaved(entry.id)}
                onCopyText={copyToClipboard}
                getAudioUrl={getAudioUrl}
                deleteAudio={deleteAudioEntry}
                retryTranscription={retryHistoryEntry}
              />
            ))}
          </div>
        </AudioPlayerGroup>
        {/* Sentinel for infinite scroll */}
        <div ref={sentinelRef} className="h-1" />
      </>
    );
  }

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <div className="space-y-2">
        <div className="px-4 flex items-center justify-between">
          <div>
            <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
              {t("settings.history.title")}
            </h2>
          </div>
          <OpenRecordingsButton
            onClick={openRecordingsFolder}
            label={t("settings.history.openFolder")}
          />
        </div>
        <div className="px-4 grid gap-3 sm:grid-cols-[1fr_auto_auto]">
          <label className="relative block">
            <Search
              aria-hidden="true"
              className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text/40"
            />
            <input
              type="search"
              value={searchQuery}
              onChange={(event) => setSearchQuery(event.target.value)}
              placeholder={t("settings.history.searchPlaceholder")}
              aria-label={t("settings.history.searchLabel")}
              className="w-full rounded-md border border-mid-gray/30 bg-background py-2 pl-9 pr-3 text-sm outline-none focus:border-logo-primary"
            />
          </label>
          <Button
            onClick={toggleNeverStore}
            variant={storageMode === "never_store" ? "primary" : "secondary"}
            size="sm"
            disabled={isUpdating("history_storage_mode")}
            className="flex items-center gap-2"
          >
            <Ban className="h-4 w-4" />
            {storageMode === "never_store"
              ? t("settings.history.neverStoreEnabled")
              : t("settings.history.neverStore")}
          </Button>
          <Button onClick={clearHistory} variant="secondary" size="sm">
            {t("settings.history.clearAll")}
          </Button>
        </div>
        <p className="px-4 text-xs text-text/50">
          {storageMode === "never_store"
            ? t("settings.history.neverStoreDescription")
            : t("settings.history.storageDescription")}
        </p>
        <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
          {content}
        </div>
      </div>
    </div>
  );
};

interface HistoryEntryProps {
  entry: HistoryEntry;
  onToggleSaved: () => void;
  onCopyText: (text: string) => void;
  getAudioUrl: (fileName: string) => Promise<string | null>;
  deleteAudio: (id: number) => Promise<void>;
  retryTranscription: (id: number) => Promise<void>;
}

const HistoryEntryComponent: React.FC<HistoryEntryProps> = ({
  entry,
  onToggleSaved,
  onCopyText,
  getAudioUrl,
  deleteAudio,
  retryTranscription,
}) => {
  const { t, i18n } = useTranslation();
  const [showCopied, setShowCopied] = useState(false);
  const [retrying, setRetrying] = useState(false);
  const [useRaw, setUseRaw] = useState(false);

  const finalText = entry.post_processed_text?.trim().length
    ? entry.post_processed_text
    : entry.transcription_text;
  const displayedText = useRaw ? entry.raw_transcript : finalText;
  const hasTranscription = displayedText.trim().length > 0;
  const rawDiffers =
    entry.raw_transcript.trim().length > 0 &&
    entry.raw_transcript.trim() !== finalText.trim();

  const handleLoadAudio = useCallback(
    () => getAudioUrl(entry.file_name),
    [getAudioUrl, entry.file_name],
  );

  const handleCopyText = () => {
    if (!hasTranscription) {
      return;
    }

    onCopyText(displayedText);
    setShowCopied(true);
    setTimeout(() => setShowCopied(false), 2000);
  };

  const handleDeleteEntry = async () => {
    if (!window.confirm(t("settings.history.deleteConfirm"))) return;
    try {
      await deleteAudio(entry.id);
    } catch (error) {
      console.error("Failed to delete entry:", error);
      toast.error(t("settings.history.deleteError"));
    }
  };

  const handleRetranscribe = async () => {
    try {
      setRetrying(true);
      await retryTranscription(entry.id);
    } catch (error) {
      console.error("Failed to re-transcribe:", error);
      toast.error(t("settings.history.retranscribeError"));
    } finally {
      setRetrying(false);
    }
  };

  const formattedDate = formatDateTime(String(entry.timestamp), i18n.language);
  const latency = entry.transcription_ms
    ? t("settings.history.latency", {
        seconds: (entry.transcription_ms / 1000).toFixed(2),
      })
    : null;
  const audioDuration = entry.audio_duration_ms
    ? t("settings.history.duration", {
        seconds: (entry.audio_duration_ms / 1000).toFixed(1),
      })
    : null;
  const speed =
    entry.audio_duration_ms && entry.audio_duration_ms > 0 && hasTranscription
      ? t("settings.history.wordsPerMinute", {
          value: Math.round(
            displayedText.trim().split(/\s+/).length /
              (entry.audio_duration_ms / 60_000),
          ),
        })
      : null;

  return (
    <div className="px-4 py-2 pb-5 flex flex-col gap-3">
      <div className="flex justify-between items-center">
        <p className="text-sm font-medium">{formattedDate}</p>
        <div className="flex items-center">
          <IconButton
            onClick={handleCopyText}
            disabled={!hasTranscription || retrying}
            title={t("settings.history.copyToClipboard")}
          >
            {showCopied ? (
              <Check width={16} height={16} />
            ) : (
              <Copy width={16} height={16} />
            )}
          </IconButton>
          <IconButton
            onClick={onToggleSaved}
            disabled={retrying}
            active={entry.saved}
            title={
              entry.saved
                ? t("settings.history.unsave")
                : t("settings.history.save")
            }
          >
            <Star
              width={16}
              height={16}
              fill={entry.saved ? "currentColor" : "none"}
            />
          </IconButton>
          <IconButton
            onClick={handleRetranscribe}
            disabled={retrying}
            title={t("settings.history.retranscribe")}
          >
            <RotateCcw
              width={16}
              height={16}
              style={
                retrying
                  ? { animation: "spin 1s linear infinite reverse" }
                  : undefined
              }
            />
          </IconButton>
          <IconButton
            onClick={handleDeleteEntry}
            disabled={retrying}
            title={t("settings.history.delete")}
          >
            <Trash2 width={16} height={16} />
          </IconButton>
        </div>
      </div>

      {(entry.application_id ||
        entry.window_title ||
        latency ||
        audioDuration) && (
        <div className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-text/50">
          {entry.application_id && <span>{entry.application_id}</span>}
          {entry.window_title && <span>{entry.window_title}</span>}
          {audioDuration && <span>{audioDuration}</span>}
          {latency && <span>{latency}</span>}
          {speed && <span>{speed}</span>}
        </div>
      )}

      <p
        className={`italic text-sm pb-2 ${
          retrying
            ? ""
            : hasTranscription
              ? "text-text/90 select-text cursor-text whitespace-pre-wrap break-words"
              : "text-text/40"
        }`}
        style={
          retrying
            ? { animation: "transcribe-pulse 3s ease-in-out infinite" }
            : undefined
        }
      >
        {retrying && (
          <style>{`
            @keyframes transcribe-pulse {
              0%, 100% { color: color-mix(in srgb, var(--color-text) 40%, transparent); }
              50% { color: color-mix(in srgb, var(--color-text) 90%, transparent); }
            }
          `}</style>
        )}
        {retrying
          ? t("settings.history.transcribing")
          : hasTranscription
            ? displayedText
            : entry.transcription_error ||
              t("settings.history.transcriptionFailed")}
      </p>

      {rawDiffers && (
        <div className="space-y-2 rounded-md border border-mid-gray/20 p-3 text-xs text-text/60">
          <div className="grid gap-3 sm:grid-cols-2">
            <div>
              <p className="mb-1 font-semibold text-text/70">
                {t("settings.history.rawTranscript")}
              </p>
              <p className="select-text whitespace-pre-wrap break-words">
                {entry.raw_transcript}
              </p>
            </div>
            <div>
              <p className="mb-1 font-semibold text-text/70">
                {t("settings.history.finalTranscript")}
              </p>
              <p className="select-text whitespace-pre-wrap break-words">
                {finalText}
              </p>
            </div>
          </div>
          <Button
            onClick={() => setUseRaw((current) => !current)}
            variant="secondary"
            size="sm"
          >
            {useRaw
              ? t("settings.history.redoCleanup")
              : t("settings.history.undoCleanup")}
          </Button>
        </div>
      )}

      <AudioPlayer onLoadRequest={handleLoadAudio} className="w-full" />
    </div>
  );
};
