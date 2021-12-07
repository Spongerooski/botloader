import type { Command } from "./Command";
import type { IntervalTimer } from "./IntervalTimer";
import type { CommandGroup } from "./CommandGroup";

export interface ScriptMeta {
  description: string;
  scriptId: number;
  commands: Array<Command>;
  commandGroups: Array<CommandGroup>;
  intervalTimers: Array<IntervalTimer>;
}
