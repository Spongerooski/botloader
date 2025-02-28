import type { PermissionOverwrite } from "./PermissionOverwrite";
import type { VideoQualityMode } from "./VideoQualityMode";

export interface VoiceChannel {
  bitrate: number;
  guild_id: string;
  id: string;
  kind: "GuildVoice" | "GuildStageVoice";
  name: string;
  parent_id: string | null;
  permission_overwrites: Array<PermissionOverwrite>;
  position: bigint;
  rtc_region: string | null;
  user_limit: number | null;
  video_quality_mode: VideoQualityMode | null;
}
