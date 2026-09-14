export interface MarkPostPublishedRequest {
  postId: string;
  revisionId: string;
  platform: string;
  publishedAt: string;
  publicationUrl: string | null;
}

export interface ManualPublicationView {
  publicationId: string;
  postId: string;
  postTitle: string;
  revisionId: string;
  platform: string;
  publishedAt: string;
  publicationUrl: string | null;
  recordedAt: string;
}

export interface PerformanceMetrics {
  impressions: number;
  reactions: number;
  comments: number;
  reposts: number;
  saves: number;
  profileViews: number;
}

export interface RecordPostPerformanceRequest {
  publicationId: string;
  metrics: PerformanceMetrics;
  notes: string;
}

export interface PerformanceRecordView {
  performanceId: string;
  publicationId: string;
  metrics: PerformanceMetrics;
  notes: string;
  recordedAt: string;
}

export interface FeedbackInsightView {
  kind: string;
  statement: string;
  sampleSize: number;
}

export interface FeedbackSnapshotView {
  publications: ManualPublicationView[];
  latestPerformance: PerformanceRecordView[];
  insights: FeedbackInsightView[];
}
