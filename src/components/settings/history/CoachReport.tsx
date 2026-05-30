import { useTranslation } from "react-i18next";

export type WordCount = { word: string; count: number };
export type CoachMetrics = {
  word_count: number;
  duration_ms: number;
  wpm: number;
  pace_band: "Slow" | "Good" | "Fast";
  fillers: WordCount[];
  filler_total: number;
  filler_rate: number;
  weak_words: WordCount[];
};

export function parseCoachMetrics(
  raw: string | null | undefined,
): CoachMetrics | null {
  if (!raw) return null;
  try {
    return JSON.parse(raw) as CoachMetrics;
  } catch {
    return null; // malformed/old data -> no report
  }
}

export function CoachReport({
  metricsJson,
}: {
  metricsJson: string | null | undefined;
}) {
  const { t } = useTranslation();
  const m = parseCoachMetrics(metricsJson);
  if (!m) return null;

  return (
    <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-text/60">
      <span>
        {m.wpm > 0 ? `${m.wpm} ${t("settings.coach.wpm")}` : "—"} ·{" "}
        {t(`settings.coach.pace.${m.pace_band.toLowerCase()}`)}
      </span>
      <span>
        {t("settings.coach.fillers")}: {m.filler_total} (
        {m.filler_rate.toFixed(1)}%)
      </span>
      {m.fillers.slice(0, 6).map((f) => (
        <span key={f.word} className="rounded bg-mid-gray/15 px-1.5 py-0.5">
          {f.word} ×{f.count}
        </span>
      ))}
    </div>
  );
}
