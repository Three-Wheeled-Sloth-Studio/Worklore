import { useEffect, useMemo, useState } from "react";
import type {
  EntityCandidateView,
  EntityReviewView,
  EntityType,
  ResolveEntityReviewRequest,
} from "../domain/types";

const ENTITY_TYPES: Array<{ value: EntityType; label: string }> = [
  { value: "employer", label: "Employer" },
  { value: "client", label: "Client" },
  { value: "project", label: "Project" },
  { value: "product", label: "Product" },
  { value: "system", label: "System" },
  { value: "repository", label: "Repository" },
  { value: "person", label: "Person" },
  { value: "location", label: "Location" },
  { value: "organization", label: "Organization" },
  { value: "email", label: "Email address" },
  { value: "phone", label: "Phone number" },
  { value: "url", label: "URL" },
  { value: "account", label: "Account" },
  { value: "identifier", label: "Identifier" },
  { value: "user_defined", label: "Other private entity" },
];

interface PrivacyReviewPanelProps {
  reviews: EntityReviewView[];
  onResolve: (request: ResolveEntityReviewRequest) => Promise<void>;
}

export function PrivacyReviewPanel({ reviews, onResolve }: PrivacyReviewPanelProps) {
  const [activeReviewId, setActiveReviewId] = useState<string | null>(
    reviews[0]?.reviewItemId ?? null,
  );
  const activeReview =
    reviews.find((review) => review.reviewItemId === activeReviewId) ?? reviews[0] ?? null;
  const [targetEntityId, setTargetEntityId] = useState<string>("");
  const [canonicalName, setCanonicalName] = useState<string>(activeReview?.matchedText ?? "");
  const [entityType, setEntityType] = useState<EntityType>(
    parseEntityType(activeReview?.suggestedEntityType),
  );
  const [submitting, setSubmitting] = useState(false);

  const existingCandidates = useMemo(
    () =>
      activeReview?.candidates.filter(
        (candidate) => candidate.entityId !== activeReview.provisionalEntityId,
      ) ?? [],
    [activeReview],
  );

  useEffect(() => {
    if (!activeReview) {
      setActiveReviewId(null);
      return;
    }
    setActiveReviewId(activeReview.reviewItemId);
    setTargetEntityId(existingCandidates[0]?.entityId ?? "");
    setCanonicalName(activeReview.matchedText);
    setEntityType(parseEntityType(activeReview.suggestedEntityType));
  }, [activeReview?.reviewItemId]);

  if (!activeReview) {
    return null;
  }

  const reviewItemId = activeReview.reviewItemId;

  async function submit(request: Omit<ResolveEntityReviewRequest, "reviewItemId">) {
    setSubmitting(true);
    try {
      await onResolve({
        reviewItemId,
        ...request,
      });
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section className="workspace-panel review-panel" aria-labelledby="privacy-review-heading">
      <div className="panel-heading-row review-heading-row">
        <div>
          <p className="eyebrow">Privacy review</p>
          <h2 id="privacy-review-heading">Resolve private names</h2>
          <p className="review-progress">
            {reviews.length} item{reviews.length === 1 ? "" : "s"} need a decision
          </p>
        </div>
        {reviews.length > 1 ? (
          <select
            aria-label="Privacy review item"
            value={activeReview.reviewItemId}
            onChange={(event) => setActiveReviewId(event.target.value)}
          >
            {reviews.map((review, index) => (
              <option value={review.reviewItemId} key={review.reviewItemId}>
                {index + 1}. {review.matchedText}
              </option>
            ))}
          </select>
        ) : null}
      </div>

      <div className="review-layout">
        <div className="review-context">
          <h3>{activeReview.question}</h3>
          <blockquote>{activeReview.contextExcerpt}</blockquote>
          <div className="confidence-row" aria-label="Detection confidence">
            <span>Detected {percent(activeReview.extractionConfidence)}</span>
            <span>Type {percent(activeReview.typeConfidence)}</span>
            <span>Match {percent(activeReview.identityMatchConfidence)}</span>
          </div>
        </div>

        <div className="review-resolution">
          {existingCandidates.length > 0 ? (
            <fieldset className="candidate-fieldset">
              <legend>Possible existing match</legend>
              {existingCandidates.map((candidate) => (
                <CandidateOption
                  candidate={candidate}
                  selected={targetEntityId === candidate.entityId}
                  onSelect={setTargetEntityId}
                  key={candidate.entityId}
                />
              ))}
            </fieldset>
          ) : (
            <p className="quiet-copy">
              WorkLore did not find a confident existing match. Confirm this as a new entity,
              adjust its type, or ignore the term.
            </p>
          )}

          <div className="review-edit-grid">
            <label>
              <span>Canonical name</span>
              <input
                value={canonicalName}
                onChange={(event) => setCanonicalName(event.target.value)}
                maxLength={500}
              />
            </label>
            <label>
              <span>Entity type</span>
              <select
                value={entityType}
                onChange={(event) => setEntityType(event.target.value as EntityType)}
              >
                {ENTITY_TYPES.map((option) => (
                  <option value={option.value} key={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <div className="review-actions">
            {targetEntityId ? (
              <>
                <button
                  className="primary-button compact"
                  disabled={submitting}
                  onClick={() =>
                    void submit({
                      action: "same_entity",
                      targetEntityId,
                      canonicalName: null,
                    })
                  }
                >
                  Same entity
                </button>
                <button
                  className="secondary-button compact"
                  disabled={submitting}
                  onClick={() =>
                    void submit({
                      action: "related_entity",
                      targetEntityId,
                      canonicalName,
                      entityType,
                    })
                  }
                >
                  Related, but separate
                </button>
              </>
            ) : null}
            <button
              className={targetEntityId ? "quiet-button compact" : "primary-button compact"}
              disabled={submitting || canonicalName.trim().length === 0}
              onClick={() =>
                void submit({
                  action: "new_entity",
                  canonicalName,
                  entityType,
                })
              }
            >
              Confirm new entity
            </button>
            <button
              className="text-button"
              disabled={submitting}
              onClick={() => void submit({ action: "ignore_term" })}
            >
              Ignore this term
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}

function CandidateOption({
  candidate,
  selected,
  onSelect,
}: {
  candidate: EntityCandidateView;
  selected: boolean;
  onSelect: (entityId: string) => void;
}) {
  return (
    <label className={`candidate-option ${selected ? "selected" : ""}`}>
      <input
        type="radio"
        name="entity-candidate"
        value={candidate.entityId}
        checked={selected}
        onChange={() => onSelect(candidate.entityId)}
      />
      <span>
        <strong>{candidate.canonicalName}</strong>
        <small>
          {candidate.publicToken} | {entityTypeLabel(candidate.entityType)} | Match {percent(candidate.score)}
        </small>
        {candidate.reasons[0] ? <small>{candidate.reasons[0]}</small> : null}
      </span>
    </label>
  );
}

function parseEntityType(value: string | undefined): EntityType {
  return ENTITY_TYPES.some((option) => option.value === value)
    ? (value as EntityType)
    : "user_defined";
}

function entityTypeLabel(value: EntityType): string {
  return ENTITY_TYPES.find((option) => option.value === value)?.label ?? "Entity";
}

function percent(value: number): string {
  return `${Math.round(value * 100)}%`;
}
