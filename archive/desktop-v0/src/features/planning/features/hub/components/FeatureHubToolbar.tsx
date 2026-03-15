import { Search } from 'lucide-react';
import { Input, Tooltip, TooltipTrigger, TooltipContent, FacetedFilter, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/primitives';
import HubViewToggle from '@/features/planning/common/hub/components/HubViewToggle';

interface FeatureHubToolbarProps {
  searchQuery: string;
  onSearchQueryChange: (value: string) => void;
  viewFilter: 'all' | 'blocking' | 'ready';
  onViewFilterChange: (value: 'all' | 'blocking' | 'ready') => void;
  statusOptions: Array<{ value: string; label: string }>;
  selectedStatuses: Set<string>;
  onSelectedStatusesChange: (values: Set<string>) => void;
  sortBy: string;
  sortOptions: Array<{ value: string; label: string }>;
  onSortByChange: (value: string) => void;
}

export default function FeatureHubToolbar({
  searchQuery,
  onSearchQueryChange,
  viewFilter,
  onViewFilterChange,
  statusOptions,
  selectedStatuses,
  onSelectedStatusesChange,
  sortBy,
  sortOptions,
  onSortByChange,
}: FeatureHubToolbarProps) {
  return (
    <div className="flex w-full flex-nowrap items-center justify-end gap-2 overflow-x-auto px-0.5 pb-0.5">
      <Tooltip>
        <TooltipTrigger asChild>
          <div className="relative h-8 w-[220px] min-w-[180px] shrink-0">
            <Search className="text-muted-foreground absolute top-1/2 left-3 size-4 -translate-y-1/2" />
            <Input
              placeholder="Search features..."
              className="h-8 pl-9"
              value={searchQuery}
              onChange={(event) => onSearchQueryChange(event.target.value)}
            />
          </div>
        </TooltipTrigger>
        <TooltipContent side="top">Search across feature titles, files, and links.</TooltipContent>
      </Tooltip>
      <div className="shrink-0">
        <HubViewToggle
          value={viewFilter}
          options={['all', 'blocking', 'ready']}
          onChange={onViewFilterChange}
        />
      </div>
      <FacetedFilter
        title="Status"
        options={statusOptions}
        selectedValues={Array.from(selectedStatuses)}
        onSelectionChange={(next: string[]) => onSelectedStatusesChange(new Set(next))}
      />
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
        <TooltipContent side="top">Order features by priority, status, or date.</TooltipContent>
      </Tooltip>
    </div>
  );
}
