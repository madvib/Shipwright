import { Input } from '@ship/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/ui';
import HubViewToggle from '@/features/planning/hub/components/HubViewToggle';

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
      <Input
        value={search}
        onChange={(event) => onSearchChange(event.target.value)}
        placeholder="Search releases"
        className="h-8 w-[210px] min-w-[180px] shrink-0"
      />
      <div className="shrink-0">
        <HubViewToggle
          value={viewFilter}
          options={['all', 'blocking', 'ready']}
          onChange={onViewFilterChange}
        />
      </div>
      <Select
        value={sortBy}
        onValueChange={(value) => {
          if (value) onSortByChange(value);
        }}
      >
        <SelectTrigger size="sm" className="w-[160px] shrink-0">
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
