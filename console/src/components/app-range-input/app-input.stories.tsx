import type { Meta, StoryObj } from "@storybook/react";

import { AppRangeInput } from "./app-range-input";
import { useState } from "react";

const meta = {
  title: "Components / App Range Input",
  component: AppRangeInput,
  tags: ["autodocs"],
  args: {
    label: "CPU Cores",
    max: 16,
    milestones: [1, 4, 8, 12, 16],
    min: 1,
    name: "cpu-input",
    onChange: () => {},
    step: 1,
    value: 4,
  },
} satisfies Meta<typeof AppRangeInput>;

export default meta;
type Story = StoryObj<typeof AppRangeInput>;

export const Primary: Story = {
  render: (args) => {
    const [value, setValue] = useState(2);

    return <AppRangeInput {...args} value={value} onChange={setValue} />;
  },
};
