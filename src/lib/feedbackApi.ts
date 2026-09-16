import { invoke } from "@tauri-apps/api/core";
import type {
  FeedbackSnapshotView,
  ManualPublicationView,
  MarkPostPublishedRequest,
  PerformanceRecordView,
  RecordPostPerformanceRequest,
} from "../domain/feedback";

export function markPostPublished(
  vaultPath: string,
  request: MarkPostPublishedRequest,
): Promise<ManualPublicationView> {
  return invoke<ManualPublicationView>("mark_post_published", { vaultPath, request });
}

export function recordPostPerformance(
  vaultPath: string,
  request: RecordPostPerformanceRequest,
): Promise<PerformanceRecordView> {
  return invoke<PerformanceRecordView>("record_post_performance", { vaultPath, request });
}

export function getFeedbackSnapshot(vaultPath: string): Promise<FeedbackSnapshotView> {
  return invoke<FeedbackSnapshotView>("get_feedback_snapshot", { vaultPath });
}
