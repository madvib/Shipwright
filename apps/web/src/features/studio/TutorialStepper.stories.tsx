import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { TutorialStepper } from './TutorialStepper'

const meta: Meta<typeof TutorialStepper> = {
  title: 'Studio/TutorialStepper',
  component: TutorialStepper,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof TutorialStepper>

/** First step active -- "Create profile" is highlighted. */
export const StepOne: Story = {
  args: {
    currentStep: 0,
    onDismiss: fn(),
  },
}

/** Second step active -- "Create profile" is done, "Add skills + MCP" is current. */
export const StepTwo: Story = {
  args: {
    currentStep: 1,
    onDismiss: fn(),
  },
}

/** Third step active -- two completed, "Wire workflow" is current. */
export const StepThree: Story = {
  args: {
    currentStep: 2,
    onDismiss: fn(),
  },
}

/** Final step active -- three completed, "Export" is current. */
export const StepFour: Story = {
  args: {
    currentStep: 3,
    onDismiss: fn(),
  },
}

/** All steps completed -- all four done. */
export const AllComplete: Story = {
  args: {
    currentStep: 4,
    onDismiss: fn(),
  },
}
