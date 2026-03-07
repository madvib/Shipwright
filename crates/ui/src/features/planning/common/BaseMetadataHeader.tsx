import { ReactNode, Fragment } from 'react';

interface BaseMetadataHeaderProps {
    children: ReactNode;
}

/**
 * A standard container for entity metadata (toolbars).
 * Automatically inserts pipe separators between top-level elements if passed as an array.
 */
export function BaseMetadataHeader({ children }: BaseMetadataHeaderProps) {
    const childrenArray = Array.isArray(children) ? children : [children];
    const filteredChildren = childrenArray.filter(Boolean);

    return (
        <div className="flex flex-nowrap items-center gap-1 overflow-hidden">
            {filteredChildren.map((child, index) => (
                <Fragment key={index}>
                    {index > 0 && <span className="text-muted-foreground/30 px-1">|</span>}
                    <div className="flex items-center">
                        {child}
                    </div>
                </Fragment>
            ))}
        </div>
    );
}
