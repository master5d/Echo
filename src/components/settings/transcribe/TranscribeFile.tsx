import { open } from "@tauri-apps/plugin-dialog";
import { type FC, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { listen } from "@tauri-apps/api/event";
import { ProgressBar } from "@/components/ui/ProgressBar";

type Format = "plain" | "inline" | "srt" | "vtt" | "json" | "karaoke";

export const TranscribeFile: FC = () => {
  const { t } = useTranslation();
  const [path, setPath] = useState<string | null>(null);
  const [timestamps, setTimestamps] = useState(false);
  const [format, setFormat] = useState<Format>("inline");
  const [diarize, setDiarize] = useState(false);
  const [speakers, setSpeakers] = useState<string>("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<{
    phase: string;
    percent: number | null;
  } | null>(null);
  const [cancelling, setCancelling] = useState(false);
  const [outputPath, setOutputPath] = useState<string | null>(null);
  const [savedTo, setSavedTo] = useState<string | null>(null);

  const pickFile = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Audio/Video",
          extensions: [
            "mp3",
            "wav",
            "m4a",
            "mp4",
            "mkv",
            "mov",
            "flac",
            "ogg",
            "webm",
          ],
        },
      ],
    });
    if (typeof selected === "string") {
      setPath(selected);
      setResult("");
      setError(null);
    }
  };

  const pickOutput = async () => {
    const { save: saveDialog } = await import("@tauri-apps/plugin-dialog");
    const ext = timestamps ? format : "txt";
    const base = path
      ? path
          .replace(/\.[^/.]+$/, "")
          .split(/[\\/]/)
          .pop()
      : "transcript";
    const target = await saveDialog({ defaultPath: `${base}.${ext}` });
    if (typeof target === "string") setOutputPath(target);
  };

  const run = async () => {
    if (!path) return;
    setBusy(true);
    setError(null);
    setResult("");
    setSavedTo(null);
    setCancelling(false);
    setProgress({ phase: "decoding", percent: null });
    const effectiveFormat: Format = timestamps ? format : "plain";
    const hint = diarize && speakers.trim() ? Number(speakers.trim()) : null;

    const unlisten = await listen<{ phase: string; percent: number | null }>(
      "transcription-progress",
      (e) => setProgress(e.payload),
    );
    const unlistenDl = await listen<{ percentage: number }>(
      "model-download-progress",
      (e) =>
        setProgress({ phase: "loading_model", percent: e.payload.percentage }),
    );
    try {
      const res = await commands.transcribeFileToString(
        path,
        null,
        null,
        diarize,
        Number.isFinite(hint) ? hint : null,
        effectiveFormat,
      );
      if (res.status === "ok") {
        setResult(res.data);
        if (outputPath) {
          const { writeTextFile } = await import("@tauri-apps/plugin-fs");
          await writeTextFile(outputPath, res.data);
          setSavedTo(outputPath);
        }
      } else if (!cancelling) setError(res.error);
    } finally {
      unlisten();
      unlistenDl();
      setBusy(false);
      setProgress(null);
    }
  };

  const cancel = async () => {
    setCancelling(true);
    await commands.cancelFileTranscription();
  };

  const save = async () => {
    const { save: saveDialog } = await import("@tauri-apps/plugin-dialog");
    const { writeTextFile } = await import("@tauri-apps/plugin-fs");
    const target = await saveDialog({
      defaultPath: `transcript.${timestamps ? format : "txt"}`,
    });
    if (typeof target === "string") await writeTextFile(target, result);
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-4 p-2">
      <h2 className="text-lg font-semibold">
        {t("settings.transcribe.title")}
      </h2>

      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={pickFile}
          className="px-3 py-2 rounded bg-indigo-600 text-white text-sm"
        >
          {t("settings.transcribe.pickFile")}
        </button>
        <span className="text-xs text-text/60 truncate">
          {path ?? t("settings.transcribe.noFile")}
        </span>
      </div>

      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={pickOutput}
          className="px-3 py-2 rounded border border-slate-700 text-sm"
        >
          {t("settings.transcribe.saveTo")}
        </button>
        <span className="text-xs text-text/60 truncate">
          {outputPath ?? ""}
        </span>
      </div>

      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={timestamps}
          onChange={(e) => setTimestamps(e.target.checked)}
        />
        {t("settings.transcribe.timestamps")}
      </label>

      {timestamps && (
        <select
          value={format}
          onChange={(e) => setFormat(e.target.value as Format)}
          className="text-sm bg-transparent border border-slate-700 rounded px-2 py-1"
        >
          <option value="inline">{t("settings.transcribe.fmtInline")}</option>
          <option value="srt">SRT</option>
          <option value="vtt">VTT</option>
          <option value="json">{t("settings.transcribe.fmtJson")}</option>
          <option value="karaoke">{t("settings.transcribe.fmtKaraoke")}</option>
        </select>
      )}

      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={diarize}
          onChange={(e) => setDiarize(e.target.checked)}
        />
        {t("settings.transcribe.diarize")}
      </label>

      {diarize && (
        <input
          type="number"
          min={1}
          placeholder={t("settings.transcribe.speakersPlaceholder")}
          value={speakers}
          onChange={(e) => setSpeakers(e.target.value)}
          className="text-sm bg-transparent border border-slate-700 rounded px-2 py-1 w-40"
        />
      )}

      {busy && progress && (
        <div className="space-y-2">
          <ProgressBar
            percent={progress.percent}
            label={t(
              `settings.transcribe.progress.${progress.phase === "loading_model" ? "loadingModel" : progress.phase}`,
            )}
          />
          <button
            type="button"
            onClick={cancel}
            disabled={cancelling}
            className="px-3 py-1.5 rounded border border-red-500/60 text-red-300 text-xs disabled:opacity-40"
          >
            {t("settings.transcribe.cancel")}
          </button>
        </div>
      )}
      {cancelling && !busy && (
        <div className="text-sm text-text/60">
          {t("settings.transcribe.cancelled")}
        </div>
      )}

      <div className="flex gap-2">
        <button
          type="button"
          disabled={!path || busy}
          onClick={run}
          className="px-3 py-2 rounded bg-green-600 text-white text-sm disabled:opacity-40"
        >
          {busy
            ? t("settings.transcribe.working")
            : t("settings.transcribe.run")}
        </button>
        {result && (
          <>
            <button
              type="button"
              onClick={() => navigator.clipboard.writeText(result)}
              className="px-3 py-2 rounded border border-slate-700 text-sm"
            >
              {t("settings.transcribe.copy")}
            </button>
            <button
              type="button"
              onClick={save}
              className="px-3 py-2 rounded border border-slate-700 text-sm"
            >
              {t("settings.transcribe.saveAs")}
            </button>
          </>
        )}
      </div>

      {error && <div className="text-sm text-red-400">{error}</div>}
      {savedTo && (
        <div className="text-sm text-green-400">
          {t("settings.transcribe.savedTo", { path: savedTo })}
        </div>
      )}
      {result && (
        <textarea
          readOnly
          value={result}
          className="w-full h-64 text-xs font-mono bg-black/30 border border-slate-800 rounded p-2"
        />
      )}
    </div>
  );
};
