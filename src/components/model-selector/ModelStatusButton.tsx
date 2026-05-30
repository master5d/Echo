import React from "react";

type ModelStatus =
  | "ready"
  | "loading"
  | "downloading"
  | "verifying"
  | "extracting"
  | "error"
  | "unloaded"
  | "none";

interface ModelStatusButtonProps {
  status: ModelStatus;
  displayText: string;
  isDropdownOpen: boolean;
  onClick: () => void;
  className?: string;
}

const ModelStatusButton: React.FC<ModelStatusButtonProps> = ({
  status,
  displayText,
  isDropdownOpen,
  onClick,
  className = "",
}) => {
  const getStatusColor = (status: ModelStatus): string => {
    switch (status) {
      case "ready":
        return "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)]";
      case "loading":
        return "bg-amber-400 animate-pulse";
      case "downloading":
        return "bg-indigo-400 animate-pulse";
      case "verifying":
        return "bg-orange-400 animate-pulse";
      case "extracting":
        return "bg-orange-400 animate-pulse";
      case "error":
        return "bg-rose-400 shadow-[0_0_8px_rgba(251,113,133,0.5)]";
      case "unloaded":
        return "bg-slate-600";
      case "none":
        return "bg-rose-400";
      default:
        return "bg-slate-600";
    }
  };

  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2.5 hover:text-indigo-300 transition-all duration-200 group ${className}`}
      title={`Model status: ${displayText}`}
    >
      <div
        className={`w-2 h-2 rounded-full transition-shadow duration-300 ${getStatusColor(status)}`}
      />
      <span className="max-w-32 truncate font-bold tracking-tight text-slate-300 group-hover:text-indigo-200">
        {displayText}
      </span>
      <svg
        className={`w-3.5 h-3.5 text-slate-500 group-hover:text-indigo-400 transition-all duration-300 ${isDropdownOpen ? "rotate-180 text-indigo-400" : ""}`}
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={3}
          d="M19 9l-7 7-7-7"
        />
      </svg>
    </button>
  );
};

export default ModelStatusButton;
