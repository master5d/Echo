import React from "react";

interface SettingsGroupProps {
  title?: string;
  description?: string;
  children: React.ReactNode;
}

export const SettingsGroup: React.FC<SettingsGroupProps> = ({
  title,
  description,
  children,
}) => {
  return (
    <div className="space-y-4 mb-8">
      {title && (
        <div className="px-5">
          <h2 className="text-[11px] font-black text-slate-500 uppercase tracking-[0.25em]">
            {title}
          </h2>
          {description && (
            <p className="text-xs text-slate-400 mt-1.5 leading-relaxed">
              {description}
            </p>
          )}
        </div>
      )}
      <div className="bg-slate-900/40 border border-slate-800/80 rounded-[2rem] overflow-hidden backdrop-blur-md shadow-2xl shadow-black/20">
        <div className="divide-y divide-slate-800/40">{children}</div>
      </div>
    </div>
  );
};
