import type { Meta, StoryObj } from '@storybook/react'
import {
  DashboardSkeleton,
  AgentListSkeleton,
  AgentDetailSkeleton,
  SkillsIdeSkeleton,
  SettingsSkeleton,
} from './StudioSkeleton'

/** All skeleton variants for Studio loading states. */
const meta: Meta = {
  title: 'Studio/Skeletons',
  parameters: { layout: 'fullscreen' },
}
export default meta

export const Dashboard: StoryObj = {
  render: () => <DashboardSkeleton />,
}

export const AgentList: StoryObj = {
  render: () => <AgentListSkeleton />,
}

export const AgentDetail: StoryObj = {
  render: () => <AgentDetailSkeleton />,
}

export const SkillsIde: StoryObj = {
  render: () => <SkillsIdeSkeleton />,
}

export const Settings: StoryObj = {
  render: () => <SettingsSkeleton />,
}
