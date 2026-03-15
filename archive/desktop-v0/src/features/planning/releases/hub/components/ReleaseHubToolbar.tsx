import { Button, Input, Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/primitives';
import { Crosshair } from 'lucide-react';
import HubViewToggle from '@/features/planning/common/hub/components/HubViewToggle';

interface ReleaseHubToolbarProps {
  search: string;
  onSearchChange: (value: string) => void;
  viewFilter: 'all' | 'blocking' | 'ready';
  onViewFilterChange: (value: 'all' | 'blocking' | 'ready') => void;
  activeTargetsOnly: boolean;
  onActiveTargetsOnlyChange: (value: boolean) => void;
  sortBy: string;
  sortOptions: Array<{ value: string; label: string }>;
  onSortByChange: (value: string) => void;
}

export default function ReleaseHubToolbar({
  search,
  onSearchChange,
  viewFilter,
  onViewFilterChange,
  activeTargetsOnly,
  onActiveTargetsOnlyChange,
  sortBy,
  sortOptions,
  onSortByChange,
}: ReleaseHubToolbarProps) {
  return (
    <div className="flex w-full flex-wrap items-center justify-end gap-2 pb-0.5">
      <Tooltip>
        <TooltipTrigger asChild>
          <div className="w-full min-w-0 sm:w-[210px] sm:min-w-[180px] sm:shrink-0">
            <Input
              value={search}
              onChange={(event) => onSearchChange(event.target.value)}
              placeholder="Search releases"
              className="h-8 w-full min-w-0"
            />
          </div>
        </TooltipTrigger>
        <TooltipContent side="top">Filter milestones by version or description.</TooltipContent>
      </Tooltip>
      <div className="shrink-0">
        <HubViewToggle
          value={viewFilter}
          options={['all', 'blocking', 'ready']}
          onChange={onViewFilterChange}
        />
      </div>
      <Tooltip delayDuration={300}>
        <TooltipTrigger asChild>
          <Button
            type="button"
            variant={activeTargetsOnly ? 'secondary' : 'outline'}
            size="sm"
            className="h-8 shrink-0 whitespace-nowrap"
            onClick={() => onActiveTargetsOnlyChange(!activeTargetsOnly)}
          >
            <Crosshair className="size-3.5" />
            Active Targets
          </Button>
        </TooltipTrigger>
        <TooltipContent side="top">
          Show only releases with current active target work.
        </TooltipContent>
      </Tooltip>
      <Tooltip delayDuration={300}>
        <TooltipTrigger asChild>
          <div className="w-full sm:w-auto sm:shrink-0">
            <Select
              value={sortBy}
              onValueChange={(value) => {
                if (value) onSortByChange(value);
              }}
            >
              <SelectTrigger size="sm" className="w-full sm:w-[160px]">
                <SelectValue>
                  {sortOptions.find((option) => option.value === sortBy)?.label}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {sortOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </TooltipTrigger>
        <TooltipContent side="top">Order releases by date, version, or progress.</TooltipContent>
      </Tooltip>
    </div>
  );
}
