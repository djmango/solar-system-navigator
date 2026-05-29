import { cn } from "@/lib/utils";

export function Badge({
  className,
  variant = "default",
  ...props
}: React.HTMLAttributes<HTMLSpanElement> & {
  variant?: "default" | "warp" | "maneuver" | "muted";
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-md px-1.5 py-0.5 text-[10px] font-medium",
        variant === "default" && "bg-sky-900/60 text-sky-200 border border-sky-700/40",
        variant === "warp" && "bg-emerald-900/50 text-emerald-300 border border-emerald-700/40",
        variant === "maneuver" && "bg-amber-900/50 text-amber-200 border border-amber-700/40",
        variant === "muted" && "bg-slate-800 text-slate-400 border border-slate-700/50",
        className,
      )}
      {...props}
    />
  );
}
