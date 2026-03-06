import * as React from "react";
import { cn } from "@/lib/utils";

export interface ButtonGroupProps extends React.HTMLAttributes<HTMLDivElement> {
    orientation?: "horizontal" | "vertical";
}

export const ButtonGroup = React.forwardRef<HTMLDivElement, ButtonGroupProps>(
    ({ className, orientation = "horizontal", ...props }, ref) => (
        <div
            ref={ref}
            className={cn(
                "inline-flex",
                orientation === "horizontal" ? "flex-row" : "flex-col",
                className
            )}
            {...props}
        />
    )
);
ButtonGroup.displayName = "ButtonGroup";

export interface ButtonGroupTextProps extends React.HTMLAttributes<HTMLSpanElement> { }

export const ButtonGroupText = React.forwardRef<HTMLSpanElement, ButtonGroupTextProps>(
    ({ className, ...props }, ref) => (
        <span
            ref={ref}
            className={cn(
                "inline-flex items-center justify-center border border-input bg-transparent px-3 text-sm",
                className
            )}
            {...props}
        />
    )
);
ButtonGroupText.displayName = "ButtonGroupText";
