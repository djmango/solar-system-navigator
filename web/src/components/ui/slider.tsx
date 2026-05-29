import * as SliderPrimitive from "@radix-ui/react-slider";
import { cn } from "@/lib/utils";

export function Slider({
  className,
  ...props
}: React.ComponentProps<typeof SliderPrimitive.Root>) {
  return (
    <SliderPrimitive.Root
      className={cn("relative flex w-full touch-none select-none items-center", className)}
      {...props}
    >
      <SliderPrimitive.Track className="relative h-1.5 w-full grow overflow-hidden rounded-full bg-slate-800">
        <SliderPrimitive.Range className="absolute h-full bg-sky-500/80" />
      </SliderPrimitive.Track>
      <SliderPrimitive.Thumb className="block h-3.5 w-3.5 rounded-full border border-sky-400/50 bg-sky-500 shadow transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/50" />
    </SliderPrimitive.Root>
  );
}
