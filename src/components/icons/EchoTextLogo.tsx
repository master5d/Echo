import React from "react";

const EchoTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <div className={`flex flex-col items-center justify-center ${className}`} style={{ width, height }}>
      <h1 className="text-4xl font-bold tracking-tighter text-indigo-500" style={{ fontFamily: "Inter, sans-serif" }}>
        ECHO
      </h1>
      <div className="mt-1 h-1 w-12 bg-indigo-400 rounded-full"></div>
    </div>
  );
};

export default EchoTextLogo;