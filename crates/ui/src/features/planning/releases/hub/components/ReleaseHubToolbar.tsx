import { Input, Tooltip, TooltipTrigger, TooltipContent } from '@ship/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/ui';
import HubViewToggle from '@/features/planning/common/hub/components/HubViewToggle';

interface ReleaseHubToolbarProps {
  search: string;
  onSearchChange: (value: string) => void;
  viewFilter: 'all' | 'blocking' | 'ready';
  onViewFilterChange: (value: 'all' | 'blocking' | 'ready') => void;
  sortBy: string;
  sortOptions: Array<{ value: string; label: string }>;
  onSortByChange: (value: string) => void;
}

export default function ReleaseHubToolbar({
  search,
  onSearchChange,
  viewFilter,
  onViewFilterChange,
  sortBy,
  sortOptions,
  onSortByChange,
}: ReleaseHubToolbarProps) {
  return (
    <div className="flex w-full flex-nowrap items-center justify-end gap-2 overflow-x-auto pb-0.5">
      <Tooltip>
        <TooltipTrigger asChild>
          <Input
            value={search}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder="Search releases"
            className="h-8 w-[210px] min-w-[180px] shrink-0"
          />
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
          <div className="shrink-0">
            <Select
              value={sortBy}
              onValueChange={(value) => {
                if (value) onSortByChange(value);
              }}
            >
              <SelectTrigger size="sm" className="w-[160px]">
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
