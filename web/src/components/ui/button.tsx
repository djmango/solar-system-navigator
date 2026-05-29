import { cva, type VariantProps } from "class-variance-authority";
import { forwardRef, type ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-1.5 rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-sky-600 text-white hover:bg-sky-500 shadow-sm",
        secondary: "bg-slate-800 text-slate-100 hover:bg-slate-700 border border-slate-600/80",
        ghost: "hover:bg-slate-800/80 text-slate-300",
        maneuver: "bg-amber-600/90 text-amber-50 hover:bg-amber-500 shadow-sm",
        destructive: "bg-red-900/80 text-red-100 hover:bg-red-800 border border-red-700/50",
        prograde: "bg-emerald-700/80 text-emerald-50 hover:bg-emerald-600 border border-emerald-600/50",
      },
      size: {
        default: "h-8 px-3 py-1.5",
        sm: "h-7 px-2 text-[11px]",
        icon: "h-8 w-8",
      },
    },
    defaultVariants: { variant: "default", size: "default" },
  },
);

export interface ButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => (
    <button ref={ref} className={cn(buttonVariants({ variant, size }), className)} {...props} />
  ),
);
Button.displayName = "Button";
