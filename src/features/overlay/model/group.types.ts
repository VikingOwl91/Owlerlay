export type Layout = "row" | "column";

export type GroupDto = {
  id: number;
  name: string;
  members: number[];
  layout: Layout;
  hide_idle: boolean;
};
