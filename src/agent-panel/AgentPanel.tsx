import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";

type QuestionEvent = {
  id: number;
  kind: "text" | "choice" | "confirm" | "notify";
  question: string;
  options: string[];
  timeout_s: number;
  speak: boolean;
  source: string;
};

export default function AgentPanel() {
  const { t } = useTranslation();
  const [q, setQ] = useState<QuestionEvent | null>(null);
  const [text, setText] = useState("");
  const [timeLeft, setTimeLeft] = useState<number | null>(null);

  useEffect(() => {
    // Cold-window race fix: pull active question on mount
    commands.agentBridgeCurrent().then((cur) => {
      if (cur) {
        setQ(cur as QuestionEvent);
        if (cur.timeout_s > 0) setTimeLeft(cur.timeout_s);
      }
    });

    const un = listen<QuestionEvent>("agent-question", (e) => {
      setQ(e.payload);
      setText("");
      if (e.payload.timeout_s > 0) setTimeLeft(e.payload.timeout_s);
      else setTimeLeft(null);
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (timeLeft === null || timeLeft <= 0 || !q) return;

    const timer = setInterval(() => {
      setTimeLeft((prev) => {
        if (prev === null || prev <= 1) {
          clearInterval(timer);
          // Timeout reached: dismiss
          commands.agentBridgeDismiss(q.id).catch(() => {});
          setQ(null);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [timeLeft, q]);

  if (!q) return null;

  const submit = (answer: string) => {
    if (q.kind !== "notify") commands.agentBridgeAnswer(q.id, answer);
    setQ(null);
  };
  const dismiss = () => {
    if (q.kind !== "notify") commands.agentBridgeDismiss(q.id);
    setQ(null);
  };

  return (
    <div className="flex h-screen flex-col gap-2 bg-zinc-900 p-3 text-zinc-100 border border-zinc-700 shadow-xl overflow-hidden">
      <div className="flex justify-between items-center">
        <div className="text-[10px] uppercase tracking-wider text-zinc-500 font-semibold">
          {t("agentPanel.from", { source: q.source })}
        </div>
        {timeLeft !== null && (
          <div className="text-[10px] text-zinc-500 font-mono">
            {t("agentPanel.timeLeft", { count: timeLeft })}
          </div>
        )}
      </div>
      <div className="text-sm font-medium leading-snug">{q.question}</div>
      {q.kind === "text" && (
        <input
          autoFocus
          className="rounded bg-zinc-800 p-2 text-sm border border-zinc-700 focus:border-zinc-500 outline-none transition-colors"
          value={text}
          placeholder={t("agentPanel.typeOrDictate")}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && text.trim() && submit(text.trim())}
        />
      )}
      {q.kind === "choice" && (
        <div className="flex flex-wrap gap-2 overflow-y-auto max-h-24 py-1">
          {q.options.map((o) => (
            <button
              key={o}
              className="rounded bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 px-3 py-1.5 text-xs transition-colors"
              onClick={() => submit(o)}
            >
              {o}
            </button>
          ))}
        </div>
      )}
      {q.kind === "confirm" && (
        <div className="flex gap-2">
          <button
            className="rounded bg-emerald-900/40 hover:bg-emerald-800/60 border border-emerald-800/50 text-emerald-200 px-4 py-1.5 text-xs transition-colors"
            onClick={() => submit("yes")}
          >
            {t("agentPanel.yes")}
          </button>
          <button
            className="rounded bg-rose-900/40 hover:bg-rose-800/60 border border-rose-800/50 text-rose-200 px-4 py-1.5 text-xs transition-colors"
            onClick={() => submit("no")}
          >
            {t("agentPanel.no")}
          </button>
        </div>
      )}
      <div className="mt-auto flex justify-end">
        <button
          className="text-[10px] text-zinc-500 hover:text-zinc-300 transition-colors uppercase tracking-tighter"
          onClick={dismiss}
        >
          {t("agentPanel.dismiss")}
        </button>
      </div>
    </div>
  );
}
