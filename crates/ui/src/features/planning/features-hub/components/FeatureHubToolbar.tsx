import { Search } from 'lucide-react';
import { Input } from '@ship/ui';
import { StatusFilter } from '@/components/app/StatusFilter';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/ui';
import HubViewToggle from '@/features/planning/hub/components/HubViewToggle';

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
    <div className="flex flex-1 flex-wrap items-center justify-end gap-2">
      <div className="relative min-w-[180px] flex-1 max-w-[280px]">
        <Search className="text-muted-foreground absolute top-1/2 left-3 size-4 -translate-y-1/2" />
        <Input
          placeholder="Search features..."
          className="pl-9 h-8"
          value={searchQuery}
          onChange={(event) => onSearchQueryChange(event.target.value)}
        />
      </div>
      <HubViewToggle
        value={viewFilter}
        options={['all', 'blocking', 'ready']}
        onChange={onViewFilterChange}
      />
      <StatusFilter
        label="Status"
        options={statusOptions}
        selectedValues={selectedStatuses}
        onSelect={onSelectedStatusesChange}
      />
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
  );
}
