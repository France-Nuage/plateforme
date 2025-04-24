import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { AppDrawer } from "./app-drawer";
import { AppButton } from "../app-button";

const meta = {
  title: "Components / App Drawer",
  component: AppDrawer,
  tags: ["autodocs"],
  args: {},
} satisfies Meta<typeof AppDrawer>;

export default meta;
type Story = StoryObj<typeof AppDrawer>;

export const Primary: Story = {
  render: (props) => {
    const [open, setOpen] = useState(true);
    return (
      <>
        <AppButton onClick={() => setOpen(!open)}>Toggle drawer</AppButton>
        <AppDrawer {...props} onClose={() => setOpen(false)} open={open} />
      </>
    );
  },
};
